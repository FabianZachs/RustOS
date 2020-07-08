# A OS built using Rust (https://os.phil-opp.com/)

## A Freestanding Rust Binary
* `#![no_std]` Disable linking to standard library


* We have `main` in rust, C, etc because the default runtime (which sets up the stack, registers, anything needed for the exeuction of the program) calls `main`. We specify `#![no_main]` to tell the compiler we do not want this default runtime, and instead use `_start` as the default entry point (for most systems).
* The linker is a program that combines the generated code into an executable. The linker assumes we are using the default runtime and builds for our host system, so we have to specify to not include this runtime. We do this by building for bare metal (could also provide arguments to the linker).
* `#![no_mangle]` so the rust compiler does not change the name of the function (in order for every function to have a unique name).

```
rustc --version --verbosee
...
host: x86_64-apple-darwin
...
```

`CPU architcure - Vendor - ABI (application binary interface)`

* ABI is like a low level API. ABI is a system interface for compiled programs (letting compilers, linkers, executables, debuggers, libraries, other object files and OS to work together).

* If we change the target to compile for, something without an OS, the linker won't try to add the standard runtime.
* Add a new target (notice the `none` for the underlying OS):
```
rustup target add thumbv7em-none-eabihf
```
* Build for that target 
```
cargo build --target thumbv7em-none-eabihf
```

### Linker Arguments
* Instead of choosing a target with no OS (so that the linker does not try to link the C runtime) we can pass arguments to the linker.
* This is linker dependent
* MacOS linker error:
```
error: linking with `cc` failed: exit code:
...
 = note: ld: entry point (_main) undefined. for architecture x86_64
          clang: error: linker command failed with exit code 1 (use -v to see invocation)
```
* In MacOS all function names have an additional `_` in the front. 
* We have to set the entry point from `main` to `_start`:
```
cargo rustc -- -C link-args="-e __start"
```

Error:
```
= note: ld: dynamic main executables must link with libSystem.dylib for architecture x86_64
```
* We need to `libSystem` library by default.
```
cargo rustc -- -C link-args="-e __start -static"
```

Error:
```
  = note: ld: library not found for -lcrt0.o
```

* MacOS by defualt link to crt0 by defult. 
```
cargo rustc -- -C link-args="-e __start -static -nostartfiles"
```

* Each platform may potentially need different build commands (the one above is for MacOS). This isn't ideal. So we add a `.cargo/config` file that contains platform specific commands.

## A Minimal Rust Kernel
### Boot process
* Computer turns on -> executes firmware from motherboard ROM (code to do power-on self-test, detects RAM, pri-initialises CPU and hardware) -> looks for bootable disk -> boots OS kernel.
* x86 firmware standards:
  1. "Basic Input/Output System" (BIOS) - old and simple
  2. "Unified Extensible Firmware Interface" (UEFI) - newer and more features

### BIOS Boot
* Almost all x86 machines support BIOS booting, even newer UEFI-based machines that use an emulated BIOS. The CPU is put into a 16-bit compatability mode (called real mode) before booting so archaic bootloaders work

* When the BIOS looks for a bootable disk, and finds one, the control is transferred to its **bootloader**. This is a 512-byte portion of executable code stored at the disk's beginning. As some bootloaders are bigger than 512 bytes, bootloaders are split into a small 512 byte first stage and a following second stage loaded by the first stage.
* The bootloader needs to:
  1. Determine the location of the kernel image on the disk and load it into memory. 
  2. Needs to switch the CPU from 16-bit **real mode** to 32-bit **protected mode** (allowing system software to use features like virtual memory, paging, safe multi-tasking) then to 64-bit **long mode** (to allow access to 64-bit instructions and registers and access to entire main memory).
  3. Query certain information (like a memory map) from the BIOS and pass it to the OS kernel.

* Writing a bootloader requires assembly and a lot of steps like "write this magic value to this register". We will use the tool **bootimage** that automatically prepends a bootloader to our kernel. 

#### The Multiboot Standard
* The Free Software Foundation wrote an open bootloader called Multiboot in 1995 so every OS doesn't need to implement it's own bootloader. This standard defines an interface between the bootloader and the OS. This allows any Multiboot compliant bootloader to load any Multiboot compliant OS. The reference implementation of the Free Software Foundation's Multiboot Specification is **GNU GRUB**, the most popular bootloader for Linux systems.

* To make a kernel Multiboot compliant, we insert a *[Multiboot header](https://www.gnu.org/software/grub/manual/multiboot/multiboot.html#OS-image-format)* at the beginning of the kernel file. With this, we can boot an OS in GRUB.
* Problems with GRUB and the Multiboot standard:
  1. They support only the 32-bit protected mode. So we have to do the CPU configuration to switch to the 64-bit long mode
  2. They are designed to make the bootloader simple, not the kernel. So the kernel needs to be linked with an adjusted default page size, otherwise GRUB can't find the Multiboot header otherwise. The [boot information](https://www.gnu.org/software/grub/manual/multiboot/multiboot.html#Boot-information-format) passed to the kernel provides lots of architecture dependent structures and not a clean abstraction.
  3. GRUB needs to be installed on the host system to create a bootable disk image from the kernel file
* Because of these drawbacks, we wont use GRUB or the Multiboot standard

### A Minimal Kernel
* We want to compile for a clearly defined target system, and not the host system.

#### Installing Rust Nightly
* To build an OS, we need to use experimental features only available on the nightly rust channel.
* We can use the nightly compiler for the current directory via `rustup override add nightly`. Or we can add the file `rust-toolchain` with the content `nightly` to the project's root directory.
* To check the nightly version is installed, run `rustc --version`. The version number should have `-nightly` at the end
* With nightly, we can opt-in for experimental features with *feature flags* at the top of our file.
* We can have an inline assembly macro (`!asm`) by adding `#![feature(asm)]` at the top of `main.rs`
* Experimental features are unstable, so future Rust versions may change/remove them

#### Target Specification
* With the `--target` parameter, Cargo supports different target systems. We describe the target by the **target triple** (CPU architecture, vendor, OS, ABI).
* Aside: Recall we had `wasm32-unknown-unknown` for WebAssembly.

* For our target system, we need some special configuration parameters (like no underlying OS). So none of the existing [target triples](https://forge.rust-lang.org/release/platform-support.html) work.
* We can define our own target via a JSON file. The JSON for the `x86_64-unknown-linux-gnu` target looks like:

```
{
    "llvm-target": "x86_64-unknown-linux-gnu",
    "data-layout": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
    "arch": "x86_64",
    "target-endian": "little",
    "target-pointer-width": "64",
    "target-c-int-width": "32",
    "os": "linux",
    "executables": true,
    "linker-flavor": "gcc",
    "pre-link-args": ["-m64"],
    "morestack": false
}
```
* Most fields are for LLVM (set of compiler and toolchain technologies to develop a front end for any programming language and a back end for any ISA) to generate code for that platform. The `data-layout` field defines the size of various integer, floating point, and pointer types.
* Other fields are for Rust for conditional compilation (like target-pointer-width).
* Other fields are to specify how the crate should be built. `pre-link-args` specifies arguments passed to the linker (takes 1+ object files -> 1 executable, or 1 library or another object file).

* We also target the `x86_64` systems with our kernel, so our target specification will be similar to the one above, placed in `x86_64-fabians_os.json`.
* We changed the `llvm-target` and `os` field to `none` since we want to run on bare metal.
* We also change from the platform's default linker, to the cross platform **LLD** linker shipped with Rust:
```
"linker-flavor": "ld.lld",
"linker": "rust-lld",
```

```
"panic-strategy": "abort",
```
* ^ specifies the target won't support stack unwinding on panic, and the program should abort directly instead. This is the same as setting `panic = "abort"` in the Cargo.toml file, so we can remove it from there.

```
"disable-redzone": true,
```
* ^ We disable a stack pointer optimization called the "red zone", because it would cause stack corruption when handling interrupts (which we will deal with since we are writing an OS). See [Disable the Red Zone](https://os.phil-opp.com/red-zone/).

```
"features": "-mmx,-sse,+soft-float",
```
* ^ this field enables/disables target features. No spaces for LLVM to correctly interpret the string, and - for disable, + for enable
* `mmx` and `sse` features determine support for SIMD instructions. Using large SIMD registers in OS kernels leads to performance problems. The kernels needs to restore registers to their original state before continuing an interrupted program. So the kernel has to save the complete SIMD state to main memory each system call or hardware interrupt. As the SIMD state is very large (512-1600 bytes) and interrupts occur often, these additional save/restore operations harm performance. We disable SIMD for our kernel, but not the applications running on top.
* The problem with disabling SIMD is that floating point operations on x86\_64 require SIMD registers by default. To solve this, we add the `soft-float` feature, which emulates all floating point operations through software functions based on normal integers (slightly slower). [Disable SIMD](https://os.phil-opp.com/disable-simd/). Rusts core libary uses floats `f32` and `f64` so we can't just not use them. We have to explicitly enable `soft-float`.

#### Building our Kernel

* We can build `cargo build --target x86_64-fabians_os.json`
* This fails, with the compiler telling us `can't find crate for 'core'`. This is because the core library is distributed with the Rust compiler as a precompiled library. So it is only valid for supported triples, not for the one we created. So we need to recompile `core` for this target

##### Cargo xbuild
* [cargo xbuild](https://github.com/rust-osdev/cargo-xbuild) is a wrapper for cargo build, which cross compiles the `core`, `compiler_builtin` and `alloc` libraries for our custom target. 
* Install is via: `cargo install cargo-xbuild`. The command depends on the rust source code, so we install it via `rustup compnent add rust-src`
* Now we can run `cargo xbuild --target x86_64-fabians_os.json`

* The `_start` entry point, called by the boot loader, is still empty.

#### Set a Default Target
* Instead of passing `--target` every time we call `cargo xbuild`, we can override the default target. In the `.cargo/config` file, put 
```
# in .cargo/config
[build]
target = "x86_64-blog_os.json"
```

* Now `cargo xbuild` runs as intended

#### Printing to Screen
* Currently, the easiest way of printing to the screen is via the [VGA text buffer](https://en.wikipedia.org/wiki/VGA_text_mode). This is a special area in memory mapped to the VGA hardware that contains the contents to displayed on screen. Normally it consists of 25 lines, each containing 80 charactercells. Each character cell displays an ASCII character with some foreground and background colors.
* To write to the buffer, we need to know it's located at `0xb8000` andthat each character cell consists of an ASCII byte and a color byte.

```rust
static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}
```
* We cast the int `0xb8000` into a raw pointer, then iterate over the bytes of the static byte string `HELLO`. Then we use the `offset` method to write the string byte and corresponding color byte
* Since we are using a raw pointer, Rust can't prove raw pointers are valid, thus we need an `unsafe` block around the memory writes. Usually, this is bad habbit. For example, we could easily write beyond the buffer's end. Instead, we'd like to write safe abstractions.

### Running our Kernel
* First, we turn our compiled kernel into a bootable disk image by linking it with a bootloader. Then we can run the disk image in the QEMU VM (or boot it on real hardware via a USB stick)

#### Creating a Bootimage
* Turning our compiled kernel into a bootable disk image requires us to link it with a bootloader. This bootloader will initialise our CPU and load our kernel.
* We can use the `bootloader` crate, which implements a basic BIOS bootloader w/t C dependencies. Just rust and inline assembly. Add the `bootloader = "0.9.3"` dependency
* Next, we need to link our kernel with the bootloader after compilation. But cargo does not support post-build scripts

* The tool `bootimage` solves this by first compiling the kernel and the bootloader, and then links them together to create a bootable disk image. `cargo install bootimage`. For this we also need the `llvm-tools-preview` rust compnent. Execute `rustup component add llvm-tools-preview`. 
* Run `cargo bootimage` which will recompile our kernel via `cargo xbuild` and will compile the bootloader. Finally, the `bootimage` will combine the bootloader and the kernel to a bottable disk image.
* This will produce a bootable disk image `target/x86_64-fabians_os/debug/bootimage-os.bin`

##### How does it work?
* `bootimage` performs the following steps:
  1. Compiles the kernel to an ELF file (Executable and Linkable format)
  2. Compiles the bootloader dependency as a standalone executable
  3. Links the bytes of the kernel ELF file to the bootloader
* When booted, the bootloader reads and parses the appended ELF file. It then maps the program segments to virtual addresses in the page table, zeros the `.bss` section and sets up the stack. Finally, it reads the entry point address (of our `_start` function) and jumps to it.

### Booting it in QEMU
* Boot it with `qemu-system-x86_64 -drive format=raw,file=target/x86_64-fabians_os/debug/bootimage-os.bin`

### Real machine
* Write it to a USB stick `dd if=target/x86_64-fabians_os/debug/bootimage-os.bin of=/dev/sdX && sync`

### Using `cargo run`
* Set the `runner` configuration key for cargo in `.cargo/config`:
```
# in .cargo/config
[target.'cfg(target_os = "none")']
runner = "bootimage runner"
```
* `target.'cfg(target_os = "none")'` means it applies to all targets that have the `"os"` field of the target configuration set to `"none"` (including ours, see `x86_64-fabians_os.json`).
* The `runner` key specifies the command should be invoked for `cargo run`, which is run after a successful build with the executable files path passed as the first argument. `bootimage runner` is specifically designed to be used as a `runner` executable. It links the given executable with the project's bootloader dependency and then launches QEMU

* Everything works with `cargo xrun` or `cargo xbuild`

## VGA Text Mode
* VGA text mode is a simple way to print text to the screen. We will make an interface that such that it's usage is safe, by encapsulating all unsafety in a seperate module.

## The VGA Text Buffer
* To display a character to the screen in VGA text mode, we need to write it to the text buffer of the VGA hardware. 
* The VGA text buffer is 2D (25x80 typically), which is directly rendered on the screen. Each entry in the array defines a single character, through the following format

Bit(s) | Value
------ | ----------------
0-7    | ASCII code point (not exactly ASCII, instead a character set named [code page 437](https://en.wikipedia.org/wiki/Code_page_437))
8-11   | Foreground color (4 bits)
12-14  | Background color (3 bits)
15     | Blink (whether the character should blink) (1 bit)

* 4 bits can be represented as 1 hex (up to 15 dec)

Number | Color      | Number + Bright Bit | Bright Color
------ | ---------- | ------------------- | -------------
0x0    | Black      | 0x8                 | Dark Gray
0x1    | Blue       | 0x9                 | Light Blue
0x2    | Green      | 0xa                 | Light Green
0x3    | Cyan       | 0xb                 | Light Cyan
0x4    | Red        | 0xc                 | Light Red
0x5    | Magenta    | 0xd                 | Pink
0x6    | Brown      | 0xe                 | Yellow
0x7    | Light Gray | 0xf                 | White

* First 3 bits of foreground color are for base color, next bit is the bright bit

* The VGA text buffer is accessible via **memory-mapped I/O** (MMIO) to address `0xb8000`. So reads and writes to that address don't access the RAM, but directly the text buffer on the VGA hardware. So we can read/write to it through normal memory operations to the address.
* Memory mapped I/O uses the same address space to address both memory and I/O devices. The memory and registers of the I/O devices are mapped to address values. So CPU instructions to access memory can also be used to access devices. Areas of the addresses used by the CPU must be reserved for I/O and not for normal physical memory. I/O devices monitor the CPU's address bus and responds to any CPU access of an address assigned to that device, connecting the data bus to the device's hardware register.
* Decoders on the I/O device (and main memory) detect when the address is in its required range.


## A Rust Module
* Let's create a rust module to handle printing: We add `mod vga_buffer` to `src/main` and create a `src/vga_buffer.rs` file.
* We create foundational structs to create a VGA writer.

* We are now able to write to the VGA buffer using the `vga_buffer::Writer`

### Volatile
* We are writing the the VGA buffer at address `0xb8000`, however, we never read from this location. The compiler doesn't know that we are accessing the VGA buffer memory (instead of normal RAM). It doesnt know that writing to this location has a side effect of printing characters onto the screen. So it may assume these writes are unnecessary, and can thus be omitted. To avoid this optimisation, we specify these writes as **volatile** (indicates the value may change between different accesses, even if it doesnt appear to be modified. It prevents the compiler from optimizing away subsequent reads/writes and thus incorrectly reusing a stale value or omitting writes). 
* We use the **volatile** crate which provides a `Volatile` wrapper type with `read` and `write` methods. Internally, these methods use the `read_volatile` abd `write_volatile` functions in the core library, thus guarenteeing the reads/writes wont be optimized away.
* We add `volatile = "0.2.6"` dependency to `cargo.toml` and update out `Buffer` struct in `vga_buffer.rs`. `Volatile<ScreenChar>` is stored in out `buffer` struct instead. `Volatile` is a wrapper that takes almost any type. Then when we write to the buffer, we have to use the `write` method (instead of doing array access).

### Formatting Macros
* Rust's formatting macros allow us to easily print different types like integers and floats. To support them, we need to implement the `core::fmt::Write` trait (a collection of methods that are required to format a message into a stream).
* The only required method of this trate is `write_str`. The parameters look similar to the ones we used to implement `write_string`, so we can just use that:

```rust
use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
```
* Now we can the `write!` macro, by passing in our `Writer` instance. Remember to `use core::fmt::Write` for the `write!` macro.

### Newlines
* When we need to start a new line, our `writer` calls `new_line`. Our implementation will, move every character one line up, (deleting the top line) and continue writing at the last line again.


## A Global Interface
* We need a global writer that can be used as an interface from other modules without carrying a `Writer` instance around. We can create a static writer. 
```rust
pub static WRITER: Writer = Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
};
```
* However, statics are initialised at compile time (vs variables which are initialised at runtime). The [const evaluator](https://rustc-dev-guide.rust-lang.org/const-eval.html). However, when compiling we get an error like:

```
error[E0015]: calls in statics are limited to constant functions, tuple structs and tuple variants
 --> src/vga_buffer.rs:7:17
  |
7 |     color_code: ColorCode::new(Color::Yellow, Color::Black),
  |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```
* `Color::new` could be used if we used a [const function](https://doc.rust-lang.org/unstable-book/language-features/const-fn.html). But the fundamental issues is:

```
error[E0396]: raw pointers cannot be dereferenced in statics
 --> src/vga_buffer.rs:8:22
  |
8 |     buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
  |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ dereference of raw pointer in constant
```
* Rust's const evaluator cannot convert raw ponters to references at compile time.

### Lazy Statics
* The [lazy\_static](https://docs.rs/lazy_static/1.0.1/lazy_static/) crate provides a `lazy_static!` macro that defines a lazily initialized `static`. The `static` lazily initializes itself when it's accessed the first time (as opposed to at compile time for cormal statics). So the initialization happens at runtime
* We add it to our `cargo.toml`

```
[dependencies.lazy_static]
version = "1.0"
features = ["spin_no_std"]
```

* We need the `spin_no_std` feature since we don't link the standard library

* Then we can use `lazy_static` to define our static `WRITER`:

```rust
// in src/vga_buffer.rs

use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER: Writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };
}
```

* But this `WRITER` is immutable, and we need it mutable to write to it. We could use a mutable static, but reads/writes would be unsafe because of potential data races. We could try using immutable statics with a `RefCell` for interior mutability. But these aren't `Sync` (types which are safe to share references between threads).

### Spinlocks
* We coud use **mutexes** for interior mutability. Threads will be blocked when the resource is already locked. But our kernel has no blocking support or even the concept of threads. So we can't use mutexes.
* A basic kind of mutex that requires no OS features: a **spinlock**. These wait in a loop while repeatedly checking if the lock is available. These use up CPU time waiting until the mutex is free.
* We use this crate to add safe [interior mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) (allows you to mutate data even where there are immutable references to that data)
* We add the dependency `spin = "0.5.2"`

```rust
// in src/vga_buffer.rs

use spin::Mutex;
...
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
```

* Now we can print directly from `_start` and no longer need to call `text_print`: `vga_buffer::WRITER.lock().write_str(...).unwrap()`.

### Safety
* The only `unsafe` block we use is to create our `Buffer` reference to `0xb80000`. All other operations are safe. Since Rust uses bounds checking for array accesses by default (recall we specified the size of the buffer with `BUFFER_HEIGHT` and `BUFFER_WIDTH`), we won't accidentally write outside of the buffer. So we have a safe interface to the outside.

### A println Macro
* We have a global writer, so can use the `println` macro anywhere in the codebase. The standard library defines `println` as:
```rust
#[macro_export] // make the macro available to the whole crate (not just the module it is defined in) and puts it at the crate root (use std::println instead of std::macros::println)
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}
```

* Then the `print!` macro is defined as:
```rust 
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}
```
* `$crate` variable ensures the macro works outside the `std` crate by expanding to `std` when used in other crates.
* We copy these two macros, and modify them to use our own `_print` function

```rust
// in src/vga_buffer.rs

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
```
* We also added the `$crate` to our `println!` so we don't need to have to import the `print!` macro if we only want `println`. (The standard library `println` used `print`)
* Since we added `#[macro_export]`, these macros are available everywhere in our crate, but also the macros are placed in the root namespace (i.e `use crate::println` and not `use crate::vga_buffers::println`)
* Since the macros need to call `_print` from outside the module, `_print` is public. But as this is a private implementation detail, we add `doc(hidden)` attribute to hide it from documentation.

* Now we can use the `println` macro in `_start`. `prinln!("HELLO FROM {}", "MACRO")`

* So far we've gone from instantiating a `Writer` on every use, to having a `lazy_static` version (which still required quite a bit of boilerplate code), to using a simple macro.

### Printing Panic Messages
* We can use this macro to print panic messages. Inside our panic handler `panic(info: &PanicInfo)`, we can call `println("{}", info)`. We can then call `panic!("Some test panic!!")` to get:
`panicked at 'Some test panic!!', src/main.rs:38:5`
