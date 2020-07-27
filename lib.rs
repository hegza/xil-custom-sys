#![no_std]
#![feature(llvm_asm)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const XPAR_XGPIOPS_0_INTR: u32 = XPS_GPIO_INT_ID;
pub const XPAR_XTTCPS_0_INTR: u32 = XPS_TTC0_0_INT_ID;
pub const XPAR_XTTCPS_1_INTR: u32 = XPS_TTC0_1_INT_ID;

/// Enable the IRQ exception.
pub unsafe fn Xil_ExceptionEnable() {
    Xil_ExceptionEnableMask(XIL_EXCEPTION_IRQ)
}
/// Disable the IRQ exception.
pub unsafe fn Xil_ExceptionDisable() {
    Xil_ExceptionDisableMask(XIL_EXCEPTION_IRQ)
}

// TODO: Xil_ExceptionEnableMask and Xil_ExceptionDisableMask can be migrated to
// the new inline assembly described in RFC 2873, but is now using the old
// LLVM inline assembly, to confirm a successful port from Xilinx own library. TODO: https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html
pub unsafe fn Xil_ExceptionEnableMask(Mask: u32) {
    let reg: u32;
    // Load from coprocessor register cpsr to `reg`
    llvm_asm!("mrs $0, cpsr" : "=r"(reg) : /* no in */ : /* no clobber */ : /* no params */ );

    // Set the flag and write back to coprocessor
    llvm_asm!("msr cpsr, $0" : /* no out */ : "r"((reg) & (!((Mask) & XIL_EXCEPTION_ALL))) : /* no clobber */ : "volatile");
}
pub unsafe fn Xil_ExceptionDisableMask(Mask: u32) {
    let reg: u32;
    // Load from coprocessor register cpsr to `reg`
    llvm_asm!("mrs $0, cpsr" : "=r"(reg) : /* no in */ : /* no clobber */ : /* no params */ );

    // Set the flag and write back to coprocessor
    llvm_asm!("msr cpsr, $0" : /* no out */ : "r"((reg) | (((Mask) & XIL_EXCEPTION_ALL))) : /* no clobber */ : "volatile");
}

/// Read interrupt status
pub unsafe fn XTtcPs_GetInterruptStatus(InstancePtr: *mut XTtcPs) -> u32 {
    InstReadReg(InstancePtr, XTTCPS_ISR_OFFSET)
}
pub unsafe fn XTtcPs_EnableInterrupts(InstancePtr: *mut XTtcPs, InterruptMask: u32) {
    InstWriteReg(
        InstancePtr,
        XTTCPS_IER_OFFSET,
        InstReadReg(InstancePtr, XTTCPS_IER_OFFSET) | InterruptMask,
    );
}
pub unsafe fn XTtcPs_SetInterval(InstancePtr: *mut XTtcPs, Value: u32) {
    InstWriteReg(InstancePtr, XTTCPS_INTERVAL_VAL_OFFSET, Value);
}
pub unsafe fn XTtcPs_ClearInterruptStatus(InstancePtr: *mut XTtcPs, InterruptMask: u32) {
    InstWriteReg(InstancePtr, XTTCPS_ISR_OFFSET as u32, InterruptMask)
}
/**
 * This function starts the counter/timer without resetting the counter
 * value.
 *
 * @param	InstancePtr is a pointer to the XTtcPs instance.
 *
 * @return	None
 *
 * @note		C-style signature:
 * void XTtcPs_Start(XTtcPs *InstancePtr)
 *
 ******************************************
 * **** */
pub unsafe fn XTtcPs_Start(InstancePtr: *mut XTtcPs) {
    InstWriteReg(
        InstancePtr,
        XTTCPS_CNT_CNTRL_OFFSET as u32,
        InstReadReg(InstancePtr, XTTCPS_CNT_CNTRL_OFFSET as u32) & !XTTCPS_CNT_CNTRL_DIS_MASK,
    )
}

unsafe fn InstReadReg(InstancePtr: *mut XTtcPs, RegOffset: u32) -> cty::c_uint {
    Xil_In32(((*InstancePtr).Config.BaseAddress + RegOffset) as *mut cty::c_uint)
}
unsafe fn InstWriteReg(InstancePtr: *mut XTtcPs, RegOffset: u32, Data: u32) {
    Xil_Out32(
        ((*InstancePtr).Config.BaseAddress + RegOffset) as *mut cty::c_uint,
        Data,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Xil_In32(Addr: *mut cty::c_uint) -> cty::c_uint {
    //let LocalAddr: *mut cty::c_uint = Addr as *mut cty::c_uint;
    ::core::ptr::read_volatile(Addr)
}
#[no_mangle]
pub unsafe extern "C" fn Xil_Out32(Addr: *mut cty::c_uint, Value: cty::c_uint) {
    //let LocalAddr: *mut cty::c_uint = Addr as *mut cty::c_uint;
    ::core::ptr::write_volatile(Addr, Value);
}
