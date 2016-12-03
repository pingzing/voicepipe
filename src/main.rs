#[macro_use]
extern crate bitflags;

extern crate user32;
extern crate winapi;
extern crate gdi32;

use std::ffi::CString;
use std::fmt;

trait Message {
    fn get_message_type(&self) -> MessageType;
    fn get_value(&self) -> u8;
    fn get_pixel_value(&self) -> u32;
}

enum MessageType {
    DataText,
    MetadataDataLength,
    MetadataEndOfMessage,
}

bitflags! {
    pub flags MessageFlags: u32 {
        const IS_METADATA_FLAG  = 0b10000000_00000000_00000000,
        const MSG_TYPE_1        = 0b01000000_00000000_00000000,
        const MSG_TYPE_2        = 0b00100000_00000000_00000000,
        const MSG_TYPE_3        = 0b00010000_00000000_00000000,
        const MSG_TYPE_4        = 0b00001000_00000000_00000000,
        const SEQ_NUM_1         = 0b00000100_00000000_00000000,
        const SEQ_NUM_2         = 0b00000010_00000000_00000000,
        const SEQ_NUM_3         = 0b00000001_00000000_00000000,
        const SEQ_NUM_4         = 0b00000000_10000000_00000000,
        const CHKSUM_1          = 0b00000000_01000000_00000000,
        const CHKSUM_2          = 0b00000000_00100000_00000000,
        const CHKSUM_3          = 0b00000000_00010000_00000000,        
        const CHKSUM_4          = 0b00000000_00001000_00000000,
        const CHKSUM_5          = 0b00000000_00000100_00000000,
        const CHKSUM_6          = 0b00000000_00000010_00000000,
        const CHKSUM_7          = 0b00000000_00000001_00000000,
        const DATA_1            = 0b00000000_00000000_10000000,
        const DATA_2            = 0b00000000_00000000_01000000,
        const DATA_3            = 0b00000000_00000000_00100000,
        const DATA_4            = 0b00000000_00000000_00010000,
        const DATA_5            = 0b00000000_00000000_00001000,
        const DATA_6            = 0b00000000_00000000_00000100,
        const DATA_7            = 0b00000000_00000000_00000010,
        const DATA_8            = 0b00000000_00000000_00000001,

        const IS_METADATA = IS_METADATA_FLAG.bits,
        const MESSAGE_TYPE = MSG_TYPE_1.bits | MSG_TYPE_2.bits 
                           | MSG_TYPE_3.bits | MSG_TYPE_4.bits,
        const SEQUENCE_NUMBER = SEQ_NUM_1.bits | SEQ_NUM_2.bits 
                              | SEQ_NUM_3.bits | SEQ_NUM_4.bits,
        const CHECKSUM = CHKSUM_1.bits | CHKSUM_2.bits | CHKSUM_3.bits 
                       | CHKSUM_4.bits | CHKSUM_5.bits | CHKSUM_6.bits 
                       | CHKSUM_7.bits,
        const DATA = DATA_1.bits | DATA_2.bits | DATA_3.bits 
                   | DATA_4.bits | DATA_5.bits | DATA_6.bits 
                   | DATA_7.bits | DATA_8.bits
    }
}

impl fmt::Display for MessageFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Is Metadata: {}, Message Type: {}, Sequence Number: {},
               Checksum: {}, Data: {}",
               self.get_is_metadata_byte(),
               self.get_message_type_byte(),
               self.get_sequence_number_byte(),
               self.get_checksum_byte(),
               self.get_data())
    }
}

impl MessageFlags {
    fn get_is_metadata_byte(&self) -> u8 {
        ((self.bits & IS_METADATA.bits) >> 23) as u8
    }

    fn get_is_metadata(&self) -> bool {
        self.get_is_metadata_byte() == 1
    }

    fn get_message_type_byte(&self) -> u8 {
        ((self.bits & MESSAGE_TYPE.bits) >> 19) as u8
    }

    fn get_message_type(&self) -> MessageType {
        let msg_type = self.get_message_type_byte();
        match self.get_is_metadata() {
            true => {
                match msg_type {
                    0x0 => MessageType::MetadataDataLength,  
                    0xF => MessageType::MetadataEndOfMessage,                  
                    _ => {
                        panic!("Invalid Metadata Message Type: {}. Bad programmer!",
                               msg_type)
                    }
                }
            }
            false => {
                match msg_type {
                    0x0 => MessageType::DataText,
                    _ => panic!("Invalid data Message Type: {}. Bad programmer!", msg_type),
                }
            }
        }
    }

    fn get_sequence_number_byte(&self) -> u8 {
        ((self.bits & SEQUENCE_NUMBER.bits) >> 15) as u8
    }

    fn get_checksum_byte(&self) -> u8 {
        ((self.bits & CHECKSUM.bits) >> 8) as u8
    }

    fn get_data(&self) -> u8 {
        (self.bits & DATA.bits) as u8
    }

    fn get_key_code(&self) -> char {
        self.get_data() as char
    }

    fn is_checksum_valid(&self) -> bool {
        // casting all to u32 here to avoid u8 wrapping behavior.
        let calculated_checksum =
            (self.get_is_metadata_byte() as u32 + self.get_message_type_byte() as u32 +
             self.get_sequence_number_byte() as u32 + self.get_data() as u32) / 4;
        if calculated_checksum as u8 == self.get_checksum_byte() {
            return true;
        } else {
            return false;
        }
    }
}

impl Message for MessageFlags {
    fn get_message_type(&self) -> MessageType {
        MessageFlags::get_message_type(self)
    }

    fn get_value(&self) -> u8 {
        MessageFlags::get_data(self)
    }

    fn get_pixel_value(&self) -> u32 {
        self.bits
    }
}

fn main() {
    let window_name = CString::new("rawr - deleteme").unwrap();
    let window_handle;
    let context_handle;
    unsafe {
        window_handle = user32::FindWindowA(std::ptr::null_mut(), window_name.as_ptr());
        context_handle = user32::GetDC(window_handle);
    }

    let mut previous_value = None;
    // various metadata flags
    let mut character_buffer: Vec<char> = vec![];

    loop {
        match read_one_message(previous_value.unwrap_or(0xFFFFFFF), context_handle) {
            Some(new_val) => {
                previous_value = Some(new_val.get_pixel_value());
                match new_val.get_message_type() {
                    MessageType::MetadataDataLength => {
                        let cap = new_val.get_value();
                        character_buffer = Vec::with_capacity(cap as usize);
                    }
                    MessageType::MetadataEndOfMessage => {
                        let message: String = character_buffer.iter().cloned().collect();
                        println!("{}", message);
                    }
                    MessageType::DataText => {
                        if character_buffer.capacity() <= character_buffer.len() {
                            continue;
                        }
                        character_buffer.push(new_val.get_value() as char);
                        if character_buffer.len() == character_buffer.capacity() {
                            let message: String = character_buffer.iter().cloned().collect();
                            println!("{}", message);
                        }
                    }
                }
            }
            None => {}
        }
    }
}


fn read_one_message(previous_value: u32, context_handle: winapi::HDC) -> Option<Box<Message>> {
    let pixel_color: u32;
    unsafe {
        pixel_color = gdi32::GetPixel(context_handle, 200, 200);
    }

    let message = MessageFlags::from_bits_truncate(pixel_color);

    if pixel_color == previous_value {
        return None;
    }

    match message.is_checksum_valid() {
        true => Some(Box::new(message)),
        false => None,
    }
}
