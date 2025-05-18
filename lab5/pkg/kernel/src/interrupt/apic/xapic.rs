use super::LocalApic;
use crate::interrupt::consts::{Interrupts, Irq};
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;

/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;

pub struct XApic {
    addr: u64,
}

impl XApic {
    pub unsafe fn new(addr: u64) -> Self {
        XApic { addr }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        unsafe { read_volatile((self.addr + reg as u64) as *const u32) }
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        unsafe {
            write_volatile((self.addr + reg as u64) as *mut u32, value);
            self.read(0x20);
        }
    }
}

impl LocalApic for XApic {
    /// If this type APIC is supported
    fn support() -> bool {
        // FIXME: Check CPUID to see if xAPIC is supported.
        CpuId::new()
            .get_feature_info()
            .map(|f| f.has_apic())
            .unwrap_or(false)
    }

    /// Initialize the xAPIC for the current CPU.
    fn cpu_init(&mut self) {
        unsafe {
            // FIXME: Enable local APIC; set spurious interrupt vector.
            bitflags! {
                struct Spiv: u32 {
                    const ENABLE = 1 << 8;// set EN bit
                    const VECTOR = Interrupts::IrqBase as u32 + Irq::Spurious as u32;
                }
            }
            let spiv_value = Spiv::ENABLE | Spiv::VECTOR;
            self.write(0xF0, spiv_value.bits());

            // FIXME: The timer repeatedly counts down at bus frequency
            bitflags! {
                struct Lvtt: u32 {
                    const PERIODIC = 1 << 17;
                    const MASKED = 1 << 16;
                    const VECTOR = Interrupts::IrqBase as u32 + Irq::Timer as u32;
                }
            }
            let lvtt_value = Lvtt::PERIODIC | Lvtt::VECTOR & !Lvtt::MASKED;
            self.write(0x320, lvtt_value.bits());

            bitflags! {
                struct Tdcr: u32 {
                    const DIVIDE_1 = 0b1011;
                    const DIVIDE_2 = 0b0000;
                    const DIVIDE_4 = 0b0001;
                    const DIVIDE_8 = 0b0010;
                    const DIVIDE_16 = 0b0011;
                    const DIVIDE_32 = 0b1000;
                    const DIVIDE_64 = 0b1001;
                    const DIVIDE_128 = 0b1010;
                }
            }
            self.write(0x3E0, Tdcr::DIVIDE_64.bits());

            bitflags! {
                struct Ticr: u32 {
                    const INIT = 0x2000;
                }
            }
            self.write(0x380, Ticr::INIT.bits());

            // FIXME: Disable logical interrupt lines (LINT0, LINT1)
            bitflags! {
                struct Lint: u32 {
                    const MASKED = 1 << 16;
                }
            }
            self.write(0x350, Lint::MASKED.bits()); //lint0
            self.write(0x360, Lint::MASKED.bits()); //lint1

            // FIXME: Disable performance counter overflow interrupts (PCINT)
            bitflags! {
            struct Pcint: u32 {
                const MASKED = 1 << 16;
            }
            }
            self.write(0x340, Pcint::MASKED.bits());

            // FIXME: Map error interrupt to IRQ_ERROR.
            bitflags! {
                struct Error: u32 {
                    const VECTOR = Interrupts::IrqBase as u32 + Irq::Error as u32;
                }
            }
            self.write(0x370, Error::VECTOR.bits());

            // FIXME: Clear error status register (requires back-to-back writes).
            self.write(0x280, 0);
            self.write(0x280, 0);

            // FIXME: Ack any outstanding interrupts.
            self.eoi();

            // FIXME: Send an Init Level De-Assert to synchronise arbitration ID's.
            self.write(0x310, 0); // set ICR 0x310
            bitflags! {
                struct Icr: u64 {
                    const BCAST = 1 << 19;
                    const INIT = 5 << 8;
                    const TMLV = 1 << 15;
                }
            }
            let icr_value = Icr::BCAST | Icr::INIT | Icr::TMLV;
            self.set_icr(icr_value.bits());

            // FIXME: Enable interrupts on the APIC (but not on the processor).
            self.write(0x1B0, 0);
        }

        // NOTE: Try to use bitflags! macro to set the flags.
    }

    fn id(&self) -> u32 {
        // NOTE: Maybe you can handle regs like `0x0300` as a const.
        unsafe { self.read(0x0020) >> 24 }
    }

    fn version(&self) -> u32 {
        unsafe { self.read(0x0030) }
    }

    fn icr(&self) -> u64 {
        unsafe { (self.read(0x0310) as u64) << 32 | self.read(0x0300) as u64 }
    }

    fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(0x0300).get_bit(12) {}
            self.write(0x0310, (value >> 32) as u32);
            self.write(0x0300, value as u32);
            while self.read(0x0300).get_bit(12) {}
        }
    }

    fn eoi(&mut self) {
        unsafe {
            self.write(0x00B0, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
