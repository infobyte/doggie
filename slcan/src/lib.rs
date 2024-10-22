#![no_std]

use embedded_can::{Frame, Id, StandardId};

fn hex_char_to_u8(hex_char: u8) -> Option<u8> {
    match hex_char {
        b'0'..=b'9' => Some(hex_char - b'0'), // Convert '0'-'9' to 0-9
        b'A'..=b'F' => Some(hex_char - b'A' + 10), // Convert 'A'-'F' to 10-15
        b'a'..=b'f' => Some(hex_char - b'a' + 10), // Convert 'a'-'f' to 10-15
        _ => None, // Handle invalid input
    }
}

fn u8slice2u32(slice: &[u8]) -> Option<u32> {
    if slice.len() > 4 {
        return None;
    }
    let mut res: u32 = 0;
    for (i, c) in slice.iter().enumerate() {
        if let Some(h) = hex_char_to_u8(*c) {
            res += (h as u32) << ((slice.len() - 1 - i) * 4);
        } else {
            return None;
        }
    }
    Some(res)
}

#[derive(Debug, Eq, PartialEq)]
pub struct CanFrame {
    id: Id,
    data: [u8; 8],
    dlc: usize, 
    is_remote: bool,
}

impl CanFrame {
    fn new_from_hex_data(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        let len = data.len();
        if len > 16 {
            return None;
        }

        if len % 2 != 0 {
            return None;
        }

        let len = len / 2;

        let mut frame = CanFrame {
            id: id.into(),
            is_remote: false,
            dlc: len,
            data: [0; 8],
        };
        
        for i in 0..len  {
            let Some(high) = hex_char_to_u8(data[2 * i]) else {
                return None;
            };

            let Some(low) = hex_char_to_u8(data[2 * i + 1]) else {
                return None;
            };

            frame.data[i] =  high << 4 | low;
        }

        Some(frame)
    }
}

impl Frame for CanFrame {
    /// Creates a new data frame.
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        if data.len() > 8 {
            return None;
        }
        let mut frame = CanFrame {
            id: id.into(),
            is_remote: false,
            dlc: data.len(), // Already asserted data.len() <= 8
            data: [0; 8],
        };
        frame.data[..data.len()].copy_from_slice(data);
        Some(frame)
    }

    /// Creates a new remote frame (RTR bit set).
    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        if dlc > 8 {
            return None;
        }
        Some(CanFrame {
            id: id.into(),
            is_remote: true,
            dlc: dlc, // Already asserted dlc <= 8
            data: [0; 8],
        })
    }

    /// Returns true if this frame is an extended frame.
    fn is_extended(&self) -> bool {
        match self.id {
            Id::Extended(_) => true,
            Id::Standard(_) => false,
        }
    }

    /// Returns true if this frame is a remote frame.
    fn is_remote_frame(&self) -> bool {
        self.is_remote
    }

    /// Returns the frame identifier.
    fn id(&self) -> Id {
        self.id
    }

    /// Returns the data length code (DLC).
    fn dlc(&self) -> usize {
        self.dlc
    }

    /// Returns the frame data.
    fn data(&self) -> &[u8] {
        &self.data[..self.dlc]
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SlcanCommand {
    OpenChannel,                                // O
    CloseChannel,                               // C
    ReadStatusFlags,                            // F
    Listen,                                     // L
    SetBitrate(SlcanBitrates),                  // S
    SetBitTimeRegister(u32),                    // s
    Frame(CanFrame),                            // t/r/T/R
    FilterId,                                   // m
    FilterMask,                                 // M
    ToggleTimestamp,                            // Z
    Version,                                    // V/v
    SerialNo,                                   // N
    IncompleteMessage
}

#[derive(Debug, Eq, PartialEq)]
pub enum SlcanError {
    InvalidCommand,
    MessageTooLong,
    CommandNotImplemented
}

#[derive(Debug, Eq, PartialEq)]
pub enum SlcanBitrates {
    CAN10KB,
    CAN20KB,
    CAN50KB,
    CAN100KB,
    CAN125KB,
    CAN250KB,
    CAN500KB,
    CAN800KB,
    CAN1000KB
}

pub struct Slcan {
    msg_buffer: [u8; 31],
    msg_len: usize,
}

impl Slcan {
    pub fn new() -> Self {
        Slcan {
            msg_buffer: [0; 31],
            msg_len: 0,
        }
    }

    pub fn from_byte(&mut self, byte: u8) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len < 31 {
            self.msg_buffer[self.msg_len] = byte;
            self.msg_len += 1;

            // Check if the buffer contains a valid command
            if byte == b'\r' {
                let cmd = self.parse_cmd();
                self.msg_len = 0;
                return cmd;
            }

            return Ok(SlcanCommand::IncompleteMessage);

        } else {
            // Message too log
            self.msg_len = 0;
            return Err(SlcanError::MessageTooLong);
        }
    }
    
    fn parse_cmd(&self) -> Result<SlcanCommand, SlcanError> {
        match self.msg_buffer[0] {
            b'O' => self.parse_open_channel(),
            b'C' => self.parse_close_channel(),
            b'F' => self.parse_status_flag(),
            b'L' => self.parse_listen(),
            b'S' => self.parse_set_bitrate(),
            b't' => self.parse_standard_frame_t(),
            _ => Err(SlcanError::CommandNotImplemented)
        }
     }

    fn parse_listen(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 2 {
            Ok(SlcanCommand::Listen)
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
    
    fn parse_status_flag(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 2 {
            Ok(SlcanCommand::ReadStatusFlags)
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
    
    fn parse_close_channel(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 2 {
            Ok(SlcanCommand::CloseChannel)
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
    
    fn parse_open_channel(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 2 {
            Ok(SlcanCommand::OpenChannel)
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
    
    fn parse_standard_frame_t(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len < 6 {
            return Err(SlcanError::InvalidCommand);
        }
        let Some(id) = u8slice2u32(&self.msg_buffer[1..4]) else {
            return Err(SlcanError::InvalidCommand);
        };
        let Some(dlc) = hex_char_to_u8(self.msg_buffer[4]) else {
            return Err(SlcanError::InvalidCommand);
        };
        
        let dlc = dlc as usize;

        if self.msg_len != dlc * 2 + 6 {
            return Err(SlcanError::InvalidCommand);
        }

        let Some(standard_id) = StandardId::new(id as u16) else {
            return Err(SlcanError::InvalidCommand);
        };

        let Some(new_frame) = CanFrame::new_from_hex_data(standard_id, &self.msg_buffer[5..5 + dlc * 2]) else {
            return Err(SlcanError::InvalidCommand);
        };

        Ok(SlcanCommand::Frame(new_frame))
    }
    
    fn parse_set_bitrate(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 3 {
            match self.msg_buffer[1] {
                b'0' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN10KB)),
                b'1' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN20KB)),
                b'2' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN50KB)),
                b'3' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN100KB)),
                b'4' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN125KB)),
                b'5' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN250KB)),
                b'6' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN500KB)),
                b'7' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN800KB)),
                b'8' => Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN1000KB)),
                _ => Err(SlcanError::InvalidCommand)
            }
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_channel_valid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'O'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::OpenChannel));
    }

    #[test]
    fn test_open_channel_invalid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'O'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_close_channel_valid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'C'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::CloseChannel));
    }

    #[test]
    fn test_close_channel_invalid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'C'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_undefined_command() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::CommandNotImplemented));
    }

    #[test]
    fn test_too_long_command() {
        let mut parser = Slcan::new();

        for _ in 0..31 {
            assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        }
        assert_eq!(parser.from_byte(b'X'), Err(SlcanError::MessageTooLong));
    }

    #[test]
    fn test_status_flag_valid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'F'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::ReadStatusFlags));
    }

    #[test]
    fn test_status_flag_invalid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'F'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_listen_valid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'L'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::Listen));
    }

    #[test]
    fn test_listen_invalid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'L'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_set_bitrate_0() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'0'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN10KB)));
    }

    #[test]
    fn test_set_bitrate_1() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN20KB)));
    }

    #[test]
    fn test_set_bitrate_2() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN50KB)));
    }

    #[test]
    fn test_set_bitrate_3() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN100KB)));
    }

    #[test]
    fn test_set_bitrate_4() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'4'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN125KB)));
    }

    #[test]
    fn test_set_bitrate_5() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'5'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN250KB)));
    }

    #[test]
    fn test_set_bitrate_6() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'6'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN500KB)));
    }

    #[test]
    fn test_set_bitrate_7() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'7'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN800KB)));
    }

    #[test]
    fn test_set_bitrate_8() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'8'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN1000KB)));
    }

    #[test]
    fn test_set_bitrate_invalid_bitrate() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'9'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_set_bitrate_invalid_command() {
        let mut parser = Slcan::new();

        assert_eq!(parser.from_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_standard_frame_t_invalid_id() {
        let mut parser = Slcan::new();
        // t1230 : can_id 0x123, can_dlc 0, no data
        assert_eq!(parser.from_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'0'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_standard_frame_t_len_0_valid_data() {
        let mut parser = Slcan::new();
        // t1230 : can_id 0x123, can_dlc 0, no data
        assert_eq!(parser.from_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'0'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::Frame(
            CanFrame {
                id: Id::Standard(StandardId::new(0x123).unwrap()), 
                data: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                dlc: 0, 
                is_remote: false,
            }
        )));
    }

    #[test]
    fn test_standard_frame_t_len_0_invalid_data() {
        let mut parser = Slcan::new();
        // t1230 : can_id 0x123, can_dlc 0, data 0x11
        assert_eq!(parser.from_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'0'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand))
    }

    #[test]
    fn test_standard_frame_t_len_3_valid_data() {
        let mut parser = Slcan::new();
        // t4563112233 : can_id 0x456, can_dlc 3, data 0x11 0x22 0x33
        assert_eq!(parser.from_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'4'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'5'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'6'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Ok(SlcanCommand::Frame(
            CanFrame {
                id: Id::Standard(StandardId::new(0x456).unwrap()), 
                data: [0x11, 0x22, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00],
                dlc: 3, 
                is_remote: false,
            }
        )));
    }

    #[test]
    fn test_standard_frame_t_len_3_invalid_data_len() {
        let mut parser = Slcan::new();
        // t4563112233 : can_id 0x456, can_dlc 3, data 0x11
        assert_eq!(parser.from_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'4'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'5'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'6'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_standard_frame_t_invalid_hex_in_len() {
        let mut parser = Slcan::new();
        // t1230 : can_id 0x123, can_dlc 0, no data
        assert_eq!(parser.from_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_standard_frame_t_invalid_len() {
        let mut parser = Slcan::new();
        // t1230 : can_id 0x123, can_dlc 0, no data
        assert_eq!(parser.from_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'f'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_standard_frame_t_invalid_hex_in_data() {
        let mut parser = Slcan::new();
        // t1230 : can_id 0x123, can_dlc 0, no data
        assert_eq!(parser.from_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.from_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }
}