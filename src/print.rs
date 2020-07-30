//! This module implements the efficient print from libxil.

/// Adds a newline (\n\r) to the format string and calls print.
#[macro_export]
macro_rules! println
{
    () => ({
        print!("\n\r")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\n\r"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\n\r"), $($args)+)
    });
}

#[macro_export]
macro_rules! print
{
    ($fmt:expr) => ({
        //crate::xil::xil_printf(concat!($fmt, "\0").as_ptr());
        xil_printf(concat!($fmt, "\0").as_ptr());
    });
    ($fmt:expr, $($args:tt)+) => ({
        // HACK: not valid, str-args need a null-terminator
        //crate::xil::xil_printf(concat!($fmt, "\0").as_ptr(), $($args)+);
        xil_printf(concat!($fmt, "\0").as_ptr(), $($args)+);
    });
}

use crate::{isdigit, outbyte, strlen};

#[no_mangle]
pub unsafe extern "C" fn xil_printf(ctrl1: *const cty::c_char, mut args: ...) {
    let mut check: i32;
    let mut dot_flag: i32;
    let mut par: params_t;

    let mut ch: cty::c_char;
    // Inserted by compiler
    //let argp: va_list;
    let mut ctrl: *const cty::c_char = ctrl1 as *const cty::c_char;

    // This is auto-inserted by compiler
    //va_start(argp, ctrl1);

    'outer: while !ctrl.is_null() && *ctrl != 0 as cty::c_char {
        /* move format string chars to buffer until a */
        /* format control is found. */
        if *ctrl != '%' as cty::c_char {
            outbyte(*ctrl);
            ctrl = ctrl.offset(1);
            continue;
        }

        /* initialize all the flags for this format. */
        dot_flag = 0;
        par = params_t {
            unsigned_flag: 0,
            left_flag: 0,
            do_padding: 0,
            pad_character: ' ' as cty::c_char,
            num2: 32767,
            num1: 0,
            len: 0,
        };

        'inner: loop {
            if !ctrl.is_null() {
                ctrl = ctrl.offset(1);
            }
            if !ctrl.is_null() {
                ch = *ctrl;
            } else {
                ch = *ctrl;
            }

            if isdigit(ch as i32) != 0 {
                if dot_flag != 0 {
                    par.num2 = getnum((&mut ctrl) as *mut *const _);
                } else {
                    if ch == '0' as cty::c_char {
                        par.pad_character = '0' as cty::c_char;
                    }
                    if !ctrl.is_null() {
                        par.num1 = getnum((&mut ctrl) as *mut *const _);
                    }
                    par.do_padding = 1;
                }
                if !ctrl.is_null() {
                    ctrl = ctrl.offset(-1);
                }
                continue 'inner;
            }

            match tolower(ch as char) {
                '%' => {
                    outbyte('%' as cty::c_char);
                    check = 1;
                }
                '-' => {
                    par.left_flag = 1;
                    check = 0;
                }
                '.' => {
                    dot_flag = 1;
                    check = 0;
                }
                'l' => {
                    check = 0;
                }
                'u' => {
                    par.unsigned_flag = 1;
                    let a: i32 = args.arg();
                    outnum(a, 10u32, &mut par);
                    check = 1;
                }
                'i' | 'd' => {
                    let a: i32 = args.arg();
                    outnum(a, 10u32, &mut par);
                    check = 1;
                }
                'p' | 'X' | 'x' => {
                    par.unsigned_flag = 1;
                    let a: i32 = args.arg();
                    outnum(a, 16u32, &mut par);
                    check = 1;
                }
                's' => {
                    let a: *const cty::c_char = args.arg();
                    outs(a, &mut par);
                    check = 1;
                }
                'c' => {
                    let a: i32 = args.arg();
                    outbyte(a as cty::c_char);
                    check = 1;
                }

                '\\' => {
                    match (*ctrl) as char {
                        'a' => {
                            outbyte(0x07 as cty::c_char);
                        }
                        'h' => {
                            outbyte(0x08 as cty::c_char);
                        }
                        'r' => {
                            outbyte(0x0D as cty::c_char);
                        }
                        'n' => {
                            outbyte(0x0D as cty::c_char);
                            outbyte(0x0A as cty::c_char);
                        }
                        _ => {
                            outbyte(*ctrl);
                        }
                    }
                    ctrl = ctrl.offset(1);
                    check = 0;
                }

                _ => {
                    check = 1;
                }
            }
            if check == 1 {
                if !ctrl.is_null() {
                    ctrl = ctrl.offset(1);
                }
                continue 'outer;
            }
        }
    }

    // This is inserted by the compiler
    //va_end(argp);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct params_t {
    pub len: i32,
    pub num1: i32,
    pub num2: i32,
    pub pad_character: cty::c_char,
    pub do_padding: i32,
    pub left_flag: i32,
    pub unsigned_flag: i32,
}

//type params_t = crate::params_s;

/// This routine gets a number from the format string.
unsafe fn getnum(linep: *mut *const cty::c_char) -> i32 {
    let mut n: i32;
    let mut result_is_digit: i32 = 0;
    let mut cptr: *const cty::c_char;

    n = 0;
    cptr = *linep;
    if !cptr.is_null() {
        result_is_digit = isdigit((*cptr) as i32);
    }
    while result_is_digit != 0 {
        if !cptr.is_null() {
            n = (n * 10) + (((*cptr) as i32) - '0' as i32);
            cptr = cptr.offset(1);
            if !cptr.is_null() {
                result_is_digit = isdigit((*cptr) as i32);
            }
        }
        result_is_digit = isdigit((*cptr) as i32);
    }
    *linep = cptr as *const cty::c_char;
    return n;
}

/// This routine moves a number to the output buffer.
unsafe fn outnum(n: i32, base: u32, par: &mut params_t) {
    let mut outbuf: [cty::c_char; 32] = core::mem::zeroed();
    const DIGITS: &[u8] = "0123456789ABCDEF".as_bytes();

    for i in 0..32 {
        outbuf[i] = '0' as cty::c_char;
    }

    let (mut num, negative) =

    /* Check if number is negative */
    if (par.unsigned_flag == 0) && (base == 10) && (n < 0) {
        (-n as u32, true)
    } else {
        (n as u32, false)
    };

    /* Build number (backwards) in outbuf */
    let mut i = 0;
    loop {
        outbuf[i] = DIGITS[(num % base) as usize];
        i += 1;
        num /= base;
        if num == 0 {
            break;
        }
    }

    if negative {
        outbuf[i] = '-' as u8;
        i += 1;
    }

    outbuf[i] = 0;
    i -= 1;

    /* Move the converted number to the buffer and */
    /* add in the padding where needed. */
    par.len = strlen(outbuf.as_ptr()) as i32;
    padding(!(par.left_flag), par);
    loop {
        outbyte(outbuf[i]);
        if i == 0 {
            break;
        }
        i -= 1;
    }
    padding(par.left_flag, par);
}

/// This routine moves a string to the output buffer as directed by the padding
/// and positioning flags.
unsafe fn outs(lp: *const cty::c_char, par: &mut params_t) {
    let mut local_ptr: *const cty::c_char = lp;

    /* pad on left if needed */
    if !local_ptr.is_null() {
        par.len = strlen(local_ptr) as i32;
    }
    padding(!par.left_flag, par);

    /* Move string to the buffer */
    while (*local_ptr != 0 as cty::c_char) && (par.num2 != 0) {
        par.num2 -= 1;
        outbyte(*local_ptr);
        local_ptr = local_ptr.offset(1);
    }

    /* Pad on right if needed */
    /* CR 439175 - elided next stmt. Seemed bogus. */
    /* par.len = strlen( lp) */
    padding(par.left_flag, par);
}

/// This routine puts pad characters into the output buffer.
unsafe fn padding(l_flag: i32, par: &params_t) {
    let mut i: i32;

    if (par.do_padding != 0) && (l_flag != 0) && (par.len < par.num1) {
        i = par.len;
        while i < par.num1 {
            outbyte(par.pad_character);
            i += 1;
        }
    }
}

fn tolower(c: char) -> char {
    c.to_ascii_lowercase()
}
