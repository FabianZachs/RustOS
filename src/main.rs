#![no_std] // Disable linking to standard library
#![no_main] // tells compiler we don't want standard runtime setup

use core::panic::PanicInfo;

/// Our own entry point (no standard runtime setup used).
/// extern "C" to tell compiler to use C calling convention.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}

/// Called on panic. We have no stack unwinding, and so, abort on panic (see Cargo.toml).
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

