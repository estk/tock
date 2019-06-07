//! Pseudo kernel-userland system call interface.
//!
//! This is for platforms that only include the "Machine Mode" privilege level.
//! Since these chips don't have hardware support for user mode, we have to fake
//! it. This means the apps have to be highly trusted as there is no real separation
//! between the kernel and apps.
//!
//! Note: this really only exists so we can demonstrate Tock running on actual
//! RISC-V hardware. Really, this is very undesirable for Tock as it violates
//! the safety properties of the OS. As hardware starts to exist that supports M
//! and U modes we will remove this.

use core::fmt::Write;
use core::ptr::{read_volatile, write_volatile};

use kernel;

#[allow(improper_ctypes)]
extern "C" {
    pub fn switch_to_user(user_stack: *const u8, process_regs: &mut [usize; 8]) -> *mut u8;
}

/// This holds all of the state that the kernel must keep for the process when
/// the process is not executing.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct RiscvimacStoredState {
    regs: [usize; 32],
    pc: usize,
}

/// Implementation of the `UserspaceKernelBoundary` for the RISC-V architecture.
pub struct SysCall();

impl SysCall {
    pub const unsafe fn new() -> SysCall {
        SysCall()
    }
}

impl kernel::syscall::UserspaceKernelBoundary for SysCall {
    type StoredState = RiscvimacStoredState;

    unsafe fn set_syscall_return_value(
        &self,
        _stack_pointer: *const usize,
        state: &mut Self::StoredState,
        return_value: isize
    ) {
        // Just need to put the return value in the a0 register for when the
        // process resumes executing.
        state.regs[9] = return_value as usize; // a0 = x10 = 9th saved register = return value


        debug!("r:{:#x} to:{:#x} ra:{:#x}", state.regs[9], state.pc, state.regs[0]);
    }

    unsafe fn set_process_function(
        &self,
        stack_pointer: *const usize,
        _remaining_stack_memory: usize,
        state: &mut RiscvimacStoredState,
        callback: kernel::procs::FunctionCall,
        first_function: bool,
        ) -> Result<*mut usize, *mut usize> {

        // Set the register state for the application when it starts
        // executing. These are the argument registers.
        state.regs[9] = callback.argument0;  // a0 = x10 = 9th saved register
        state.regs[10] = callback.argument1; // a1 = x11 = 10th saved register
        state.regs[11] = callback.argument2; // a2 = x12 = 11th saved register
        state.regs[12] = callback.argument3; // a3 = x13 = 12th saved register

        // We also need to set the return address (ra) register so that the
        // new function that the process is running returns to the correct
        // location. However, if this is the first time the process is running
        // then there is nothing to return to so we skip this.
        if !first_function {
            state.regs[0] = state.pc;        // ra = x1 = 1st saved register
        }

        // Save the PC we expect to execute.
        state.pc = callback.pc;

        debug!("going to {:#x}, ra: {:#x}", callback.pc, state.regs[0]);

        Ok(stack_pointer as *mut usize)
    }

    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        _state: &mut RiscvimacStoredState,
        ) -> (*mut usize, kernel::syscall::ContextSwitchReason) {
        let mut switchReason: u32;
        switchReason = 0;
        let mut syscall0: u32;
        let mut syscall1: u32;
        let mut syscall2: u32;
        let mut syscall3: u32;
        let mut syscall4: u32;
        let mut newsp: u32;


        asm! ("
          // Before switching to the app we need to save the kernel registers to
          // the kernel stack. We then save the stack pointer in the mscratch
          // CSR (0x340) so we can retrieve it after returning to the kernel
          // from the app.

          addi sp, sp, -31*4  // Move the stack pointer down to make room.

          sw   x1, 0*4(sp)    // Save all of the registers on the kernel stack.
          sw   x3, 1*4(sp)
          sw   x4, 2*4(sp)
          sw   x5, 3*4(sp)
          sw   x6, 4*4(sp)
          sw   x7, 5*4(sp)
          sw   x8, 6*4(sp)
          sw   x9, 7*4(sp)
          sw   x10, 8*4(sp)
          sw   x11, 9*4(sp)
          sw   x12, 10*4(sp)
          sw   x13, 11*4(sp)
          sw   x14, 12*4(sp)
          sw   x15, 13*4(sp)
          sw   x16, 14*4(sp)
          sw   x17, 15*4(sp)
          sw   x18, 16*4(sp)
          sw   x19, 17*4(sp)
          sw   x20, 18*4(sp)
          sw   x21, 19*4(sp)
          sw   x22, 20*4(sp)
          sw   x23, 21*4(sp)
          sw   x24, 22*4(sp)
          sw   x25, 23*4(sp)
          sw   x26, 24*4(sp)
          sw   x27, 25*4(sp)
          sw   x28, 26*4(sp)
          sw   x29, 27*4(sp)
          sw   x30, 28*4(sp)
          sw   x31, 29*4(sp)

          sw $8, 30*4(sp)     // Store process state pointer on stack as well.
                              // We need to have the available for after the app
                              // returns to the kernel so we can store its
                              // registers.

          csrw 0x340, sp      // Save stack pointer in mscratch. This allows
                              // us to find it when the app returns back to
                              // the kernel.

          // Read current mstatus CSR and then modify it so we switch to
          // user mode when running the app.
          csrr t0, 0x300      // Read mstatus=0x300 CSR
          // Set the mode to user mode and set MPIE.
          li   t1, 0x1808     // t1 = MSTATUS_MPP & MSTATUS_MIE
          not  t1, t1         // t1 = ~(MSTATUS_MPP & MSTATUS_MIE)
          and  t0, t0, t1     // t0 = mstatus & ~(MSTATUS_MPP & MSTATUS_MIE)
          ori  t0, t0, 0x80   // t0 = t0 | MSTATUS_MPIE
          csrw 0x300, t0      // Set mstatus CSR so that we switch to user mode.

          // We have to set the mepc CSR with the PC we want the app to start
          // executing at. This has been saved in RiscvimacStoredState for us
          // (either when the app returned back to the kernel or in the
          // `set_process_function()` function).
          lw   t0, 32*4($8)   // Retrieve the PC from RiscvimacStoredState
          csrw 0x341, t0      // Set mepc CSR. This is the PC we want to go to.

          // Setup the stack pointer for the application.
          add  x2, x0, $7     // Set sp register with app stack pointer.

          // Restore all of the app registers from what we saved. If this is the
          // first time running the app then most of these values are
          // irrelevant, However we do need to set the four arguments to the
          // `_start_ function in the app. If the app has been executing then this
          // allows the app to correctly resume.
          lw   x1, 0*4($8)
          lw   x3, 2*4($8)
          lw   x4, 3*4($8)
          lw   x5, 4*4($8)
          lw   x6, 5*4($8)
          lw   x7, 6*4($8)
          lw   x8, 7*4($8)
          lw   x9, 8*4($8)
          lw   x10, 9*4($8)   // a0
          lw   x11, 10*4($8)  // a1
          lw   x12, 11*4($8)  // a2
          lw   x13, 12*4($8)  // a3
          lw   x14, 13*4($8)
          lw   x15, 14*4($8)
          lw   x16, 15*4($8)
          lw   x17, 16*4($8)
          lw   x18, 17*4($8)
          lw   x19, 18*4($8)
          lw   x20, 19*4($8)
          lw   x21, 20*4($8)
          lw   x22, 21*4($8)
          lw   x23, 22*4($8)
          lw   x24, 23*4($8)
          lw   x25, 24*4($8)
          lw   x26, 25*4($8)
          lw   x27, 26*4($8)
          lw   x28, 27*4($8)
          lw   x29, 28*4($8)
          lw   x30, 29*4($8)
          lw   x31, 30*4($8)

          // Call mret to jump to where mepc points, switch to user mode, and
          // start running the app.
          mret




          // This is where the trap handler jumps back to after the app stops
          // executing.
        _return_to_kernel:

          // We can read mcause out of the mscratch CSR because the trap handler
          // stored it there for us. We need to use mcause to determine why the
          // app stopped executing and handle it appropriately.
          csrr t0, 0x340      // CSR=0x340=mscratch
          // If mcause < 0 then we encountered an interrupt.
          blt  t0, x0, _app_interrupt // If negative, this was an interrupt.


          // Check the various exception codes and handle them properly.

          andi  t0, t0, 0x1ff // `and` mcause with 9 lower bits of zero
                              // to mask off just the cause. This is needed
                              // because the E21 core uses several of the upper
                              // bits for other flags.

        _check_ecall_umode:
          li    t1, 8          // 8 is the index of ECALL from U mode.
          beq   t0, t1, _done // Check if we did an ECALL and handle it
                               // correctly.


          // ~~
          // other exception checks go here
          // ~~
            
         
          // An interrupt occurred while the app was running.
          // TODO
        _app_interrupt:
          // li $0, 1      //set app_interrupt to 1   
          j _ecall


        // _some_other_exception:
        //   li $0, 2      //set app_interrupt to 1   
        //   j _ecall


          // Fall through to error.
          j _go_red

          // Stop here if we get here. This means there was some other exception that
          // we are not handling. The red LED will come on.
        _go_red:
          lui t5, 0x20002
          addi t5, t5, 0x00000008
          li t6, 0x00000007
          sw t6, 0(t5)
          lui t5, 0x20002
          addi t5, t5, 0x0000000c
          li t6, 0x1
          sw t6, 0(t5)
          j _go_red
       

        _done:
          // We have to get the values that the app passed to us in registers
          // (these are stored in RiscvimacStoredState) and copy them to
          // registers so we can use them when returning to the kernel loop.
          lw $1, 9*4($8)      // Fetch a0
          lw $2, 10*4($8)     // Fetch a1
          lw $3, 11*4($8)     // Fetch a2
          lw $4, 12*4($8)     // Fetch a3
          lw $5, 13*4($8)     // Fetch a4
          lw $6, 1*4($8)      // Fetch sp

          j _ecall


        _ecall:
          // Need to increment the PC so when we return we start at the correct
          // instruction. The hardware does not do this for us.
          lw   t0, 32*4($8)   // Get the PC from RiscvimacStoredState
          addi t0, t0, 4      // Add 4 to increment the PC past ecall instruction
          sw   t0, 32*4($8)   // Save the new PC back to RiscvimacStoredState

          //j _done





          "
          : "=r"(switchReason), "=r" (syscall0), "=r" (syscall1), "=r" (syscall2), "=r" (syscall3), "=r" (syscall4), "=r" (newsp)
          : "r"(stack_pointer), "r"(_state)
          : "a0", "a1", "a2", "a3"
          : "volatile");


        debug!("syscall: {:#x} {:#x} {:#x} {:#x} {:#x} {:#x}",
            syscall0, syscall1, syscall2, syscall3, syscall4, newsp);

        // (
        //     newsp as *mut usize,
        //     kernel::syscall::ContextSwitchReason::Fault
        //     )

        let syscall = kernel::syscall::arguments_to_syscall(
            syscall0 as u8, syscall1 as usize, syscall2 as usize, syscall3 as usize, syscall4 as usize);

        let mut ret: kernel::syscall::ContextSwitchReason;
        if (switchReason == 1){
            //debug_gpio!(1, set);
            ret = kernel::syscall::ContextSwitchReason::Interrupted;
            switchReason = 0;
        }
        else if (switchReason == 2){
            ret = kernel::syscall::ContextSwitchReason::Fault;
            switchReason = 0;
        }
        // // else if(syscall.is_some()){
        //     ret = kernel::syscall::ContextSwitchReason::SyscallFired{syscall: syscall};
        // }
        // else{
        //     ret = kernel::syscall::ContextSwitchReason::Fault;
        // }
        else{
            ret = match syscall {
            Some(s) => kernel::syscall::ContextSwitchReason::SyscallFired{
                syscall: s
            },
            None => kernel::syscall::ContextSwitchReason::Fault
        };

        }


        (newsp as *mut usize, ret)
    }

    unsafe fn fault_fmt(&self, writer: &mut Write) {}

    unsafe fn process_detail_fmt(
        &self,
        stack_pointer: *const usize,
        state: &RiscvimacStoredState,
        writer: &mut Write,
        ) {
    }
}
