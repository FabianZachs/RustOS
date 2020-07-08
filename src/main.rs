#![no_std] // Disable linking to standard library
#![no_main] // tells compiler we don't want standard runtime setup

use core::panic::PanicInfo;

mod vga_buffer;

static HELLO: &[u8] = b"HELLO WORLD!!";
const CYAN: u8 = 0xb;

/// Our own entry point (no standard runtime setup used).
/// Linker looks for a function named `_start` by default
/// extern "C" to tell compiler to use C calling convention.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    //vga_buffer::test_print();
    vga_buffer::draw_pattern();
    vga_buffer::WRITER
        .lock()
        .write_str("HELLO FROM MAIN")
        .unwrap();
    use core::fmt::Write;
    write!(
        vga_buffer::WRITER.lock(),
        "\nSOME MORE TEXT!! {}",
        3.0 / 2.0
    )
    .unwrap();
    println!("HELLO FROM {}", "MACRO");
    //    let vga_buffer = 0xb8000 as *mut u8; // raw pointer to u8
    //
    //    for (i, &byte) in HELLO.iter().enumerate() {
    //        unsafe {
    //            *vga_buffer.offset(i as isize * 2) = byte;
    //            *vga_buffer.offset(i as isize * 2 + 1) = CYAN;
    //        }
    //    }
    panic!("Some test panic!!");
    loop {}
}

/// Called on panic. We have no stack unwinding, and so, abort on panic (see Cargo.toml -> Now see x86_64-fabians_os.json).
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
