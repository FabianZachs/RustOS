#![no_std] // Disable linking to standard library
#![no_main] // tells compiler we don't want standard runtime setup

use core::panic::PanicInfo;

static HELLO: &[u8] = b"HELLO WORLD!!";
const CYAN: u8 = 0xb;

/// Our own entry point (no standard runtime setup used).
/// Linker looks for a function named `_start` by default
/// extern "C" to tell compiler to use C calling convention.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8; // raw pointer to u8

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = CYAN;
        }
    }
    loop {}
}

/// Called on panic. We have no stack unwinding, and so, abort on panic (see Cargo.toml -> Now see x86_64-fabians_os.json).
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

