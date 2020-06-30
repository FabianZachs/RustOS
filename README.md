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
