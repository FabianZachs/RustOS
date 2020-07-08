use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Red, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // to enable Copy semantics
#[repr(u8)] // to store each element as a byte (Rust does not have a u4 type)
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// To represent a full color code that specifies the foreground and background color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // the layout and ABI of the whole struct is guaranteed to be the same as that one field (u8)
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// Structure to represent a screen character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // to ensure the same field ordering
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

/// Structure to represent the text VGA buffer.
/// 2D array of ScreenChars.
/// We use the volatile crate as a wrapper of ScreenChars
#[repr(transparent)] // to ensure the struct has the same memory layout as its single field
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Used to write to the screen.
/// Need this to be public for external access.
/// Always writes to the last line and shifts lines up when a line is full (or on \n)
pub struct Writer {
    column_position: usize, // current position
    color_code: ColorCode,  // current fg and bg colors
    buffer: &'static mut Buffer, // reference to the VGA buffer. The 'static lifetime specifies
                            // that the reference is valid for enture duration of program
}

impl Writer {
    fn write_string(&mut self, s: &str) {
        for ascii_character in s.bytes() {
            match ascii_character {
                0x20..=0x7e | b'\n' => self.write_byte(ascii_character),
                _ => self.write_byte(0xfe), // not part of valid range (print ■)
            }
        }
    }

    pub fn write_byte(&mut self, ascii_character: u8) {
        match ascii_character {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                //self.buffer.chars[row][col] = ScreenChar {
                //    ascii_character,
                //    color_code,
                //};
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn draw_pattern() {
    //let mut writer = Writer {
    //    column_position: 0,
    //    color_code: ColorCode::new(Color::Pink, Color::White),
    //    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    //};
    for _row in 0..BUFFER_HEIGHT {
        for _col in 0..BUFFER_WIDTH {
            //writer.write_byte(0xfe);
            WRITER.lock().write_byte(0xfe);
        }
    }
}

//pub fn test_print() {
//    use core::fmt::Write;
//    let mut writer = Writer {
//        column_position: 0,
//        color_code: ColorCode::new(Color::White, Color::Red),
//        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//    };
//    writer.write_byte(b'H');
//    writer.write_string("ellö ");
//    write!(
//        writer,
//        "We can do cool complicated things like {} and {} now",
//        4,
//        3. / 2.0
//    )
//    .unwrap();
//}
