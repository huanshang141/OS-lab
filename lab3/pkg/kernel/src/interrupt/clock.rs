use super::consts::*;
use crate::as_handler;
use crate::memory::gdt::TIMER_IST_INDEX;
use crate::proc::context;
use crate::proc::switch;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler)
        .set_stack_index(TIMER_IST_INDEX);
}

pub extern "C" fn clock(mut context: context::ProcessContext) {
    switch(&mut context);
}

as_handler!(clock);
