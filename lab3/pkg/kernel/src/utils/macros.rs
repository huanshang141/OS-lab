use crate::drivers::serial::SERIAL; //not found in this scope
use crate::drivers::serial::get_serial;
use core::fmt::*;
use x86_64::instructions::interrupts;

/// Use spin mutex to control variable access
#[macro_export]
macro_rules! guard_access_fn {
    ($(#[$meta:meta])* $v:vis $fn:ident ($mutex:path : $ty:ty)) => {
        paste::item! {

            $(#[$meta])*
            #[inline(never)]
            #[allow(non_snake_case, dead_code)]
            $v fn $fn<'a>() -> Option<spin::MutexGuard<'a, $ty>> {
                $mutex.get().and_then(spin::Mutex::try_lock)
            }

            $(#[$meta])*
            #[inline(never)]
            #[allow(non_snake_case, dead_code)]
            $v fn [< $fn _for_sure >]<'a>() -> spin::MutexGuard<'a, $ty> {
                $mutex.get().and_then(spin::Mutex::try_lock).expect(
                    stringify!($mutex has not been initialized or lockable)
                )
            }
        }
    };
}

#[macro_export]
macro_rules! once_mutex {
    ($i:vis $v:ident: $t:ty) => {
        $i static $v: spin::Once<spin::Mutex<$t>> = spin::Once::new();

        paste::item! {
            #[allow(non_snake_case)]
            $i fn [<init_ $v>]([<val_ $v>]: $t) {
                $v.call_once(|| spin::Mutex::new([<val_ $v>]));
            }
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::utils::print_internal(format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n\r"));
    ($($arg:tt)*) => ($crate::print!("{}\n\r", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn print_internal(args: Arguments) {
    interrupts::without_interrupts(|| {
        if let Some(mut serial) = get_serial() {
            serial.write_fmt(args).unwrap();
        }
    });
}

#[allow(dead_code)]
#[cfg_attr(target_os = "none", panic_handler)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // force unlock serial for panic output
    unsafe { SERIAL.get().unwrap().force_unlock() };

    //llm assist
    println!("\n\r==================================================");
    println!("!!! KERNEL PANIC !!!");
    println!("==================================================");
    println!("An unrecoverable error has occurred in the kernel");
    println!("Error details: {}", info);

    // Print location information if available
    if let Some(location) = info.location() {
        println!(
            "Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    println!("--------------------------------------------------");
    println!("CPU halted. System needs to be restarted manually.");
    println!("==================================================\n\r");

    // Enter infinite loop
    loop {
        x86_64::instructions::hlt();
    }
}
