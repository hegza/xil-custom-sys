#![no_std]
#![feature(asm)]
// Allow C-style conventions
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![allow(clippy::redundant_static_lifetimes)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const XPAR_XGPIOPS_0_INTR: u32 = XPS_GPIO_INT_ID;
pub const XPAR_XTTCPS_0_INTR: u32 = XPS_TTC0_0_INT_ID;
pub const XPAR_XTTCPS_1_INTR: u32 = XPS_TTC0_1_INT_ID;

/// Enable the IRQ exception.
///
/// # Safety
/// Writing to a register is unsafe.
pub unsafe fn Xil_ExceptionEnable() {
    Xil_ExceptionEnableMask(XIL_EXCEPTION_IRQ)
}
/// Disable the IRQ exception.
///
/// # Safety
/// Writing to a register is unsafe.
pub unsafe fn Xil_ExceptionDisable() {
    Xil_ExceptionDisableMask(XIL_EXCEPTION_IRQ)
}

/// # Safety
/// Writing to a register is unsafe.
pub unsafe fn Xil_ExceptionEnableMask(Mask: u32) {
    let mut v: u32;
    // Load cpsr from coprocessor register to `v`
    asm!(
        "mrs {v}, cpsr",
        v = out(reg) v,
    );

    // Set the exception mode mask
    v &= !((Mask) & XIL_EXCEPTION_ALL);

    // Write back to coprocessor
    asm!(
        "msr cpsr, {v}",
        v = in(reg) v,
    );
}
/// # Safety
/// Writing to a register is unsafe.
pub unsafe fn Xil_ExceptionDisableMask(Mask: u32) {
    let mut v: u32;
    // Load cpsr from coprocessor register to `v`
    asm!(
        "mrs {v}, cpsr",
        v = out(reg) v,
    );

    // Unset the exception mode mask
    v |= (Mask) & XIL_EXCEPTION_ALL;

    // Write back to coprocessor
    asm!(
        "msr cpsr, {v}",
        v = in(reg) v,
    );
}

/// Read interrupt status
///
/// # Safety
/// Passing a null-ptr as `InstancePtr`is undefined behavior.
pub unsafe fn XTtcPs_GetInterruptStatus(InstancePtr: *mut XTtcPs) -> u32 {
    InstReadReg(InstancePtr, XTTCPS_ISR_OFFSET)
}
/// # Safety
/// Passing a null-ptr as `InstancePtr`is undefined behavior.
pub unsafe fn XTtcPs_EnableInterrupts(InstancePtr: *mut XTtcPs, InterruptMask: u32) {
    InstWriteReg(
        InstancePtr,
        XTTCPS_IER_OFFSET,
        InstReadReg(InstancePtr, XTTCPS_IER_OFFSET) | InterruptMask,
    );
}
/// # Safety
/// Passing a null-ptr as `InstancePtr`is undefined behavior.
pub unsafe fn XTtcPs_SetInterval(InstancePtr: *mut XTtcPs, Value: u32) {
    InstWriteReg(InstancePtr, XTTCPS_INTERVAL_VAL_OFFSET, Value);
}
/// # Safety
/// Passing a null-ptr as `InstancePtr`is undefined behavior.
pub unsafe fn XTtcPs_ClearInterruptStatus(InstancePtr: *mut XTtcPs, InterruptMask: u32) {
    InstWriteReg(InstancePtr, XTTCPS_ISR_OFFSET as u32, InterruptMask)
}
/**
 * This function starts the counter/timer without resetting the counter
 * value.
 *
 * @param   InstancePtr is a pointer to the XTtcPs instance.
 *
 * @return  None
 *
 * @note    C-style signature:
 * void XTtcPs_Start(XTtcPs *InstancePtr)
 *
 * # Safety
 * Passing a null-ptr as `InstancePtr`is undefined behavior.
 **************************************
 * **** */
pub unsafe fn XTtcPs_Start(InstancePtr: *mut XTtcPs) {
    InstWriteReg(
        InstancePtr,
        XTTCPS_CNT_CNTRL_OFFSET as u32,
        InstReadReg(InstancePtr, XTTCPS_CNT_CNTRL_OFFSET as u32) & !XTTCPS_CNT_CNTRL_DIS_MASK,
    )
}

/// # Safety
/// Passing a null-ptr as `InstancePtr`is undefined behavior.
unsafe fn InstReadReg(InstancePtr: *mut XTtcPs, RegOffset: u32) -> cty::c_uint {
    Xil_In32(((*InstancePtr).Config.BaseAddress + RegOffset) as *mut cty::c_uint)
}

/// # Safety
/// Writing to a register is unsafe. Passing a null-ptr as `InstancePtr`is
/// undefined behavior.
unsafe fn InstWriteReg(InstancePtr: *mut XTtcPs, RegOffset: u32, Data: u32) {
    Xil_Out32(
        ((*InstancePtr).Config.BaseAddress + RegOffset) as *mut cty::c_uint,
        Data,
    );
}

/// # Safety
/// Passing a null-ptr as `InstancePtr`is undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn Xil_In32(Addr: *mut cty::c_uint) -> cty::c_uint {
    ::core::ptr::read_volatile(Addr)
}

/// # Safety
/// Writing to a register is unsafe.
#[no_mangle]
pub unsafe extern "C" fn Xil_Out32(Addr: *mut cty::c_uint, Value: cty::c_uint) {
    ::core::ptr::write_volatile(Addr, Value);
}
