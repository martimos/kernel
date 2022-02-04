use core::fmt;

use bootloader::boot_info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use font8x8::UnicodeFonts;
use spin::Mutex;

static WRITER: Mutex<Writer> = Mutex::new(Writer::new());

pub fn init_vga_buffer(buffer: &'static mut FrameBuffer) {
    WRITER.lock().init(buffer);
}

#[derive(Default)]
pub struct Writer<'a> {
    x_pos: usize,
    y_pos: usize,
    buffer: &'a mut [u8],
    info: Option<FrameBufferInfo>,
}

impl<'a> fmt::Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl<'a> Writer<'a> {
    const fn new() -> Self {
        Writer {
            x_pos: 0,
            y_pos: 0,
            buffer: &mut [],
            info: None,
        }
    }

    /// Initializes this writer with the given framebuffer.
    /// This writer will write into the framebuffer.
    /// This is unsafe because the caller must ensure that
    /// the frame buffer is valid.
    pub fn init(&mut self, buffer: &'static mut FrameBuffer) {
        self.info = Some(buffer.info());
        self.buffer = buffer.buffer_mut();
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' | b'\t' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if self.buffer.is_empty() {
            panic!("vga buffer not initialized");
        }

        match byte {
            b'\n' => self.new_line(),
            b'\xFE' => {
                // for some reason, this is a backspace
            }
            b'\t' => {
                for _ in 0..4 {
                    self.write_byte(b' ');
                }
            }
            _ => {
                let c = font8x8::BASIC_FONTS
                    .get(byte as char)
                    .expect("no matching character found");
                for (y, row_byte) in c.iter().enumerate() {
                    for (x, col_bit) in (0..8).enumerate() {
                        let alpha = if *row_byte & (1 << col_bit) == 0 {
                            0
                        } else {
                            255
                        };
                        self.write_pixel(self.x_pos + x, self.y_pos + y, alpha);
                    }
                }
                self.x_pos += 8;
            }
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.unwrap().stride + x;
        let color = match self.info.unwrap().pixel_format {
            PixelFormat::RGB => [intensity, intensity, intensity / 2, 0],
            PixelFormat::BGR => [intensity / 2, intensity, intensity, 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            _ => unreachable!(),
        };
        let bytes_per_pixel = self.info.unwrap().bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.buffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { core::ptr::read_volatile(&self.buffer[byte_offset]) };
    }

    pub fn clear_screen(&mut self) {
        self.buffer.fill(0);
    }

    fn new_line(&mut self) {
        let line_height = 8 + 4;
        self.x_pos = 0;
        self.y_pos += line_height;

        // check if we need to "scroll"
        if self.y_pos + 8 >= self.info.unwrap().vertical_resolution {
            self.y_pos -= line_height;
            let line_pixel_count = line_height
                * self.info.unwrap().bytes_per_pixel
                * self.info.unwrap().horizontal_resolution;

            // clear the first line
            self.buffer[1..line_pixel_count].fill(0);

            // rotate screen buffer, making the first (cleared) line the last
            self.buffer.rotate_left(line_pixel_count);
        }
    }
}

#[macro_export]
macro_rules! vga_print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vga_println {
    () => ($crate::vga_print!("\n"));
    ($($arg:tt)*) => ($crate::vga_print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vga_clear {
    () => {
        $crate::vga_buffer::_clear()
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // disable interrupts while holding a lock on the WRITER
    // so that no deadlock can occur when we want to print
    // something in an interrupt handler
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _clear() {
    use x86_64::instructions::interrupts;

    // disable interrupts while holding a lock on the WRITER
    // so that no deadlock can occur when we want to print
    // something in an interrupt handler
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen();
    });
}
