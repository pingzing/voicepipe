extern crate user32;
extern crate winapi;
extern crate gdi32;

use std::ffi::CString;
use std::fmt;
use std::io;
use std::io::Write;

struct PixelMessage {
    original_value: u32,
    sequence_number: u8,
    checksum: u8,
    key_code: u8,
    is_virtual_key_code: bool,
    is_ctrl_down: bool,
    is_alt_down: bool,
    is_shift_down: bool,
    is_win_down: bool,
}

impl PixelMessage {
    fn is_checksum_valid(&self) -> bool {
        //casting all to u32 here to avoid u8 wrapping behavior.
        let calculated_checksum =
            (self.sequence_number as u32
            + self.key_code as u32
            + self.is_ctrl_down as u32
            + self.is_alt_down as u32
            + self.is_shift_down as u32 
            + self.is_win_down as u32) 
            / 6;                         
        if calculated_checksum as u8 == self.checksum {            
            return true;
        } else {
            return false;
        }
    }
}

fn main() {
    let window_name = CString::new("main.rs").unwrap();

    let window_handle;
    let context_handle;
    unsafe {
        window_handle = user32::FindWindowA(std::ptr::null_mut(), window_name.as_ptr());
        context_handle = user32::GetDC(window_handle);
    }

    let mut previous_value: Option<u32> = None;

    loop {
        let pixel_color: u32;
        unsafe {
            pixel_color = gdi32::GetPixel(context_handle, 200, 200);
        }

        if previous_value.is_some() && pixel_color == previous_value.unwrap() {
            continue;
        }        

        //println!("Binary: {:b}", pixel_color);

        let message = PixelMessage {
            original_value: pixel_color,
            //per WinAPI's definition, COLORREF, which we get back from GetPixel, is a 3 byte value and the first byte is always zeroed.
            sequence_number:        ((pixel_color & 0b11110000_00000000_00000000u32) >> 20) as u8, // top 4 bits of a 24-bit value            
            checksum:               ((pixel_color & 0b00001111_11100000_00000000u32) >> 13) as u8, /* next 7 bits */
            key_code:               ((pixel_color & 0b00000000_00011111_11100000u32) >> 5) as u8, /* next 8 bits */
            is_virtual_key_code:    ((pixel_color & 0b00000000_00000000_00010000u32) >> 4) == 1, // next 1 bit
            is_ctrl_down:           ((pixel_color & 0b00000000_00000000_00001000u32) >> 3) == 1, /* next 1 bit */
            is_alt_down:            ((pixel_color & 0b00000000_00000000_00000100u32) >> 2) == 1, /* next 1 bit */
            is_shift_down:          ((pixel_color & 0b00000000_00000000_00000010u32) >> 1) == 1, // next 1 bit
            is_win_down:            ((pixel_color & 0b00000000_00000000_00000001u32)) == 1, /* next 1 bit */
        };          
        
        // compare checksum
        if message.is_checksum_valid() {
            print!("{}", message.key_code as char);
            io::stdout().flush().ok().expect("Could not flush stdout");         
        }

        previous_value = Some(pixel_color);
    }
}

impl fmt::Display for PixelMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "sequence_number: {}, checksum: {}, keycode: {}, 
               is_virtual_key_code: {}, is_ctrl_down:{},
               is_alt_down: {}, is_shift_down: {}, is_win_down: {}",
               self.sequence_number,
               self.checksum,
               self.key_code,
               self.is_virtual_key_code,
               self.is_ctrl_down,
               self.is_alt_down,
               self.is_shift_down,
               self.is_win_down)
    }
}

impl fmt::Debug for PixelMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
        "Binary: {:b},
        {}", self.original_value, self)
    }
}
