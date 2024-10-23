#![no_std]

use embedded_can::{Id, ExtendedId, StandardId};

fn nibble_to_hex_char(value: u8) -> u8 {
    match value {
        0..=9 => b'0' + value,
        10..=15 => b'A' + (value - 10),
        _ => unreachable!(), // Since we only pass nibbles (0-15), this case is unreachable
    }
}


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

// impl Frame for CanFrame {
//     /// Creates a new data frame.
//     fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
//         if data.len() > 8 {
//             return None;
//         }
//         let mut frame = CanFrame {
//             id: id.into(),
//             is_remote: false,
//             dlc: data.len(), // Already asserted data.len() <= 8
//             data: [0; 8],
//         };
//         frame.data[..data.len()].copy_from_slice(data);
//         Some(frame)
//     }

//     /// Creates a new remote frame (RTR bit set).
//     fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
//         if dlc > 8 {
//             return None;
//         }
//         Some(CanFrame {
//             id: id.into(),
//             is_remote: true,
//             dlc: dlc, // Already asserted dlc <= 8
//             data: [0; 8],
//         })
//     }

//     /// Returns true if this frame is an extended frame.
//     fn is_extended(&self) -> bool {
//         match self.id {
//             Id::Extended(_) => true,
//             Id::Standard(_) => false,
//         }
//     }

//     /// Returns true if this frame is a remote frame.
//     fn is_remote_frame(&self) -> bool {
//         self.is_remote
//     }

//     /// Returns the frame identifier.
//     fn id(&self) -> Id {
//         self.id
//     }

//     /// Returns the data length code (DLC).
//     fn dlc(&self) -> usize {
//         self.dlc
//     }

//     /// Returns the frame data.
//     fn data(&self) -> &[u8] {
//         &self.data[..self.dlc]
//     }
// }

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

pub struct SlcanSerializer {
    msg_buffer: [u8; 31],
    msg_len: usize,
}

impl SlcanSerializer {
    pub fn new() -> Self {
        SlcanSerializer {
            msg_buffer: [0; 31],
            msg_len: 0,
        }
    }

    pub fn to_bytes(&mut self, cmd: SlcanCommand) -> Option<[u8; 31]> {
        if let SlcanCommand::Frame(frame) = cmd {
            match frame.is_remote {
                true => return Some(self.serialize_frame_r(frame)),
                false => return Some(self.serialize_frame_t(frame))
            }
        }
        None
    }

    fn serialize_frame_r(&mut self, frame: CanFrame) -> [u8; 31] {
        let mut res = [0; 31];
        match frame.id {
            Id::Standard(id) => {
                res[0] = b'r';
                res[1] = nibble_to_hex_char(((id.as_raw() >> 8) & 0xf) as u8);
                res[2] = nibble_to_hex_char(((id.as_raw() >> 4) & 0xf) as u8);
                res[3] = nibble_to_hex_char((id.as_raw() & 0xf) as u8);
                res[4] = nibble_to_hex_char(frame.dlc as u8);

                let mut i = 0;

                while i < frame.dlc {
                    res[5 + 2 * i] = nibble_to_hex_char(((frame.data[i] >> 4) & 0xf) as u8);
                    res[6 + 2 * i] = nibble_to_hex_char((frame.data[i] & 0xf) as u8);
                    i += 1;
                }

                res[5 + 2 * i] = b'\r'
            }
            Id::Extended(id) => {
                res[0] = b'R';
                res[1] = nibble_to_hex_char(((id.as_raw() >> 28) & 0xf) as u8);
                res[2] = nibble_to_hex_char(((id.as_raw() >> 24) & 0xf) as u8);
                res[3] = nibble_to_hex_char(((id.as_raw() >> 20) & 0xf) as u8);
                res[4] = nibble_to_hex_char(((id.as_raw() >> 16) & 0xf) as u8);
                res[5] = nibble_to_hex_char(((id.as_raw() >> 12) & 0xf) as u8);
                res[6] = nibble_to_hex_char(((id.as_raw() >> 8) & 0xf) as u8);
                res[7] = nibble_to_hex_char(((id.as_raw() >> 4) & 0xf) as u8);
                res[8] = nibble_to_hex_char((id.as_raw() & 0xf) as u8);
                res[9] = nibble_to_hex_char(frame.dlc as u8);

                let mut i = 0;

                while i < frame.dlc {
                    res[10 + 2 * i] = nibble_to_hex_char(((frame.data[i] >> 4) & 0xf) as u8);
                    res[11 + 2 * i] = nibble_to_hex_char((frame.data[i] & 0xf) as u8);
                    i += 1;
                }

                res[10 + 2 * i] = b'\r'
            }
        }
        res
    }

    fn serialize_frame_t(&mut self, frame: CanFrame) -> [u8; 31] {
        let mut res = [0; 31];
        match frame.id {
            Id::Standard(id) => {
                res[0] = b't';
                res[1] = nibble_to_hex_char(((id.as_raw() >> 8) & 0xf) as u8);
                res[2] = nibble_to_hex_char(((id.as_raw() >> 4) & 0xf) as u8);
                res[3] = nibble_to_hex_char((id.as_raw() & 0xf) as u8);
                res[4] = nibble_to_hex_char(frame.dlc as u8);

                let mut i = 0;

                while i < frame.dlc {
                    res[5 + 2 * i] = nibble_to_hex_char(((frame.data[i] >> 4) & 0xf) as u8);
                    res[6 + 2 * i] = nibble_to_hex_char((frame.data[i] & 0xf) as u8);
                    i += 1;
                }

                res[5 + 2 * i] = b'\r'
            }
            Id::Extended(id) => {
                res[0] = b'T';
                res[1] = nibble_to_hex_char(((id.as_raw() >> 28) & 0xf) as u8);
                res[2] = nibble_to_hex_char(((id.as_raw() >> 24) & 0xf) as u8);
                res[3] = nibble_to_hex_char(((id.as_raw() >> 20) & 0xf) as u8);
                res[4] = nibble_to_hex_char(((id.as_raw() >> 16) & 0xf) as u8);
                res[5] = nibble_to_hex_char(((id.as_raw() >> 12) & 0xf) as u8);
                res[6] = nibble_to_hex_char(((id.as_raw() >> 8) & 0xf) as u8);
                res[7] = nibble_to_hex_char(((id.as_raw() >> 4) & 0xf) as u8);
                res[8] = nibble_to_hex_char((id.as_raw() & 0xf) as u8);
                res[9] = nibble_to_hex_char(frame.dlc as u8);

                let mut i = 0;

                while i < frame.dlc {
                    res[10 + 2 * i] = nibble_to_hex_char(((frame.data[i] >> 4) & 0xf) as u8);
                    res[11 + 2 * i] = nibble_to_hex_char((frame.data[i] & 0xf) as u8);
                    i += 1;
                }

                res[10 + 2 * i] = b'\r'
            }
        }
        res
    }
    
    pub fn from_bytes(&mut self, bytes: &[u8]) -> Result<SlcanCommand, SlcanError> {
        for byte in bytes.iter() {
            let res = self.from_byte(*byte);
            if res == Ok(SlcanCommand::IncompleteMessage) {
                continue;
            } else {
                return res;
            }
        };
        Err(SlcanError::InvalidCommand)
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
            b'O' => self.deserialize_open_channel(),
            b'C' => self.deserialize_close_channel(),
            b'F' => self.deserialize_status_flag(),
            b'L' => self.deserialize_listen(),
            b'S' => self.deserialize_set_bitrate(),
            b's' => Err(SlcanError::CommandNotImplemented),
            b't' => self.deserialize_standard_frame_t(),
            b'T' => Err(SlcanError::CommandNotImplemented),
            b'r' => Err(SlcanError::CommandNotImplemented),
            b'R' => Err(SlcanError::CommandNotImplemented),
            b'm' => Err(SlcanError::CommandNotImplemented),
            b'M' => Err(SlcanError::CommandNotImplemented),
            b'Z' => Err(SlcanError::CommandNotImplemented),
            b'V' => Err(SlcanError::CommandNotImplemented),
            b'v' => Err(SlcanError::CommandNotImplemented),
            b'N' => Err(SlcanError::CommandNotImplemented),
            _ => Err(SlcanError::InvalidCommand)
        }
     }

    fn deserialize_listen(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 2 {
            Ok(SlcanCommand::Listen)
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
    
    fn deserialize_status_flag(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 2 {
            Ok(SlcanCommand::ReadStatusFlags)
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
    
    fn deserialize_close_channel(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 2 {
            Ok(SlcanCommand::CloseChannel)
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
    
    fn deserialize_open_channel(&self) -> Result<SlcanCommand, SlcanError> {
        if self.msg_len == 2 {
            Ok(SlcanCommand::OpenChannel)
        } else {
            Err(SlcanError::InvalidCommand)
        }
    }
    
    fn deserialize_standard_frame_t(&self) -> Result<SlcanCommand, SlcanError> {
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
    
    fn deserialize_set_bitrate(&self) -> Result<SlcanCommand, SlcanError> {
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
    fn test_deserialize_from_bytes() {
        let mut serializer = SlcanSerializer::new();
        let _ = serializer.from_byte(b'O');
        let cmd_from_byte = serializer.from_byte(b'\r');
        let cmd_from_bytes = serializer.from_bytes(b"O\r");
        assert_eq!(cmd_from_byte, cmd_from_bytes);
    }

    #[test]
    fn test_deserialize_from_bytes_invalid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"O"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_open_channel_valid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"O\r"), Ok(SlcanCommand::OpenChannel));
    }

    #[test]
    fn test_deserialize_open_channel_invalid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"OX\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_close_channel_valid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"C\r"), Ok(SlcanCommand::CloseChannel));
    }

    #[test]
    fn test_deserialize_close_channel_invalid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"CX\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_command_not_implemented() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"s\r"), Err(SlcanError::CommandNotImplemented));
    }

    #[test]
    fn test_deserialize_undefined_command() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"X\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_too_long_command() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"),
            Err(SlcanError::MessageTooLong)
        );
    }

    #[test]
    fn test_deserialize_status_flag_valid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"F\r"), Ok(SlcanCommand::ReadStatusFlags));
    }

    #[test]
    fn test_deserialize_status_flag_invalid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"FX\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_listen_valid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"L\r"), Ok(SlcanCommand::Listen));
    }

    #[test]
    fn test_deserialize_listen_invalid() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"LX\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_set_bitrate_0() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S0\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN10KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_1() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S1\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN20KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_2() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S2\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN50KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_3() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S3\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN100KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_4() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S4\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN125KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_5() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S5\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN250KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_6() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S6\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN500KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_7() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S7\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN800KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_8() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S8\r"),
            Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN1000KB))
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_invalid_bitrate() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S9\r"),
            Err(SlcanError::InvalidCommand)
        );
    }

    #[test]
    fn test_deserialize_set_bitrate_invalid_command() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(
            serializer.from_bytes(b"S\r"),
            Err(SlcanError::InvalidCommand)
        );
    }

    #[test]
    fn test_deserialize_standard_frame_t_invalid_id_no_hex() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"t12X0\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_standard_frame_t_invalid_id_too_high() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"tfff0\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_standard_frame_t_len_0_valid_data() {
        let mut serializer = SlcanSerializer::new();
        // t1230 : can_id 0x123, can_dlc 0, no data
        assert_eq!(
            serializer.from_bytes(b"t1230\r"),
            Ok(SlcanCommand::Frame(
                CanFrame {
                    id: Id::Standard(StandardId::new(0x123).unwrap()), 
                    data: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 0, 
                    is_remote: false,
                }
            ))
        );
    }

    #[test]
    fn test_deserialize_standard_frame_t_len_0_invalid_data() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"t123011\r"), Err(SlcanError::InvalidCommand))
    }

    #[test]
    fn test_deserialize_standard_frame_t_len_3_valid_data() {
        let mut serializer = SlcanSerializer::new();
        // t4563112233 : can_id 0x456, can_dlc 3, data 0x11 0x22 0x33
        assert_eq!(
            serializer.from_bytes(b"t4563112233\r"),
            Ok(SlcanCommand::Frame(
                CanFrame {
                    id: Id::Standard(StandardId::new(0x456).unwrap()), 
                    data: [0x11, 0x22, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 3, 
                    is_remote: false,
                }
            ))
        );
    }

    #[test]
    fn test_deserialize_standard_frame_t_len_3_invalid_data_len() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"t456311\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_standard_frame_t_invalid_hex_in_len() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"t123X\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_standard_frame_t_invalid_len() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"t123f11\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_standard_frame_t_invalid_hex_in_data_high() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"t1231X1\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_standard_frame_t_invalid_hex_in_data_low() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"t12311X\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_deserialize_standard_frame_t_too_short() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.from_bytes(b"t123\r"), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_serialize_command_not_frame() {
        let mut serializer = SlcanSerializer::new();
        assert_eq!(serializer.to_bytes(SlcanCommand::OpenChannel), None)
    }

    #[test]
    fn test_serialize_standard_frame_t_len_0() {
        let mut serializer = SlcanSerializer::new();
        let mut res: [u8; 31] = [0; 31];
        res[0] = b't';
        res[1] = b'1';
        res[2] = b'2';
        res[3] = b'3';
        res[4] = b'0';
        res[5] = b'\r';

        assert_eq!(
            serializer.to_bytes(
                SlcanCommand::Frame(CanFrame { 
                    id: Id::Standard(StandardId::new(0x123).unwrap()),
                    data: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 0,
                    is_remote: false
                })
            ).unwrap(),
            res
        );
    }

    #[test]
    fn test_serialize_standard_frame_t_len_3() {
        let mut serializer = SlcanSerializer::new();
        let mut res: [u8; 31] = [0; 31];
        res[0] = b't';
        res[1] = b'1';
        res[2] = b'2';
        res[3] = b'3';
        res[4] = b'3';
        res[5] = b'F';
        res[6] = b'1';
        res[7] = b'F';
        res[8] = b'2';
        res[9] = b'F';
        res[10] = b'3';
        res[11] = b'\r';

        assert_eq!(
            serializer.to_bytes(
                SlcanCommand::Frame(CanFrame { 
                    id: Id::Standard(StandardId::new(0x123).unwrap()),
                    data: [0xf1, 0xf2, 0xf3, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 3,
                    is_remote: false
                })
            ).unwrap(),
            res
        );
    }


    #[test]
    fn test_serialize_extended_frame_t_len_0() {
        let mut serializer = SlcanSerializer::new();
        let mut res: [u8; 31] = [0; 31];
        res[0] = b'T';
        res[1] = b'1';
        res[2] = b'2';
        res[3] = b'3';
        res[4] = b'4';
        res[5] = b'5';
        res[6] = b'6';
        res[7] = b'7';
        res[8] = b'8';
        res[9] = b'0';
        res[10] = b'\r';

        assert_eq!(
            serializer.to_bytes(
                SlcanCommand::Frame(CanFrame { 
                    id: Id::Extended(ExtendedId::new(0x12345678).unwrap()),
                    data: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 0,
                    is_remote: false
                })
            ).unwrap(),
            res
        );
    }

    #[test]
    fn test_serialize_extended_frame_t_len_3() {
        let mut serializer = SlcanSerializer::new();
        let mut res: [u8; 31] = [0; 31];
        res[0] = b'T';
        res[1] = b'1';
        res[2] = b'2';
        res[3] = b'3';
        res[4] = b'4';
        res[5] = b'5';
        res[6] = b'6';
        res[7] = b'7';
        res[8] = b'8';
        res[9] = b'3';
        res[10] = b'F';
        res[11] = b'1';
        res[12] = b'F';
        res[13] = b'2';
        res[14] = b'F';
        res[15] = b'3';
        res[16] = b'\r';

        assert_eq!(
            serializer.to_bytes(
                SlcanCommand::Frame(CanFrame { 
                    id: Id::Extended(ExtendedId::new(0x12345678).unwrap()),
                    data: [0xf1, 0xf2, 0xf3, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 3,
                    is_remote: false
                })
            ).unwrap(),
            res
        );
    }

    #[test]
    fn test_serialize_standard_frame_r_len_0() {
        let mut serializer = SlcanSerializer::new();
        let mut res: [u8; 31] = [0; 31];
        res[0] = b'r';
        res[1] = b'1';
        res[2] = b'2';
        res[3] = b'3';
        res[4] = b'0';
        res[5] = b'\r';

        assert_eq!(
            serializer.to_bytes(
                SlcanCommand::Frame(CanFrame { 
                    id: Id::Standard(StandardId::new(0x123).unwrap()),
                    data: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 0,
                    is_remote: true
                })
            ).unwrap(),
            res
        );
    }

    #[test]
    fn test_serialize_standard_frame_r_len_3() {
        let mut serializer = SlcanSerializer::new();
        let mut res: [u8; 31] = [0; 31];
        res[0] = b'r';
        res[1] = b'1';
        res[2] = b'2';
        res[3] = b'3';
        res[4] = b'3';
        res[5] = b'F';
        res[6] = b'1';
        res[7] = b'F';
        res[8] = b'2';
        res[9] = b'F';
        res[10] = b'3';
        res[11] = b'\r';

        assert_eq!(
            serializer.to_bytes(
                SlcanCommand::Frame(CanFrame { 
                    id: Id::Standard(StandardId::new(0x123).unwrap()),
                    data: [0xf1, 0xf2, 0xf3, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 3,
                    is_remote: true
                })
            ).unwrap(),
            res
        );
    }

    #[test]
    fn test_serialize_extended_frame_r_len_0() {
        let mut serializer = SlcanSerializer::new();
        let mut res: [u8; 31] = [0; 31];
        res[0] = b'R';
        res[1] = b'1';
        res[2] = b'2';
        res[3] = b'3';
        res[4] = b'4';
        res[5] = b'5';
        res[6] = b'6';
        res[7] = b'7';
        res[8] = b'8';
        res[9] = b'0';
        res[10] = b'\r';

        assert_eq!(
            serializer.to_bytes(
                SlcanCommand::Frame(CanFrame { 
                    id: Id::Extended(ExtendedId::new(0x12345678).unwrap()),
                    data: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 0,
                    is_remote: true
                })
            ).unwrap(),
            res
        );
    }

    #[test]
    fn test_serialize_extended_frame_r_len_3() {
        let mut serializer = SlcanSerializer::new();
        let mut res: [u8; 31] = [0; 31];
        res[0] = b'R';
        res[1] = b'1';
        res[2] = b'2';
        res[3] = b'3';
        res[4] = b'4';
        res[5] = b'5';
        res[6] = b'6';
        res[7] = b'7';
        res[8] = b'8';
        res[9] = b'3';
        res[10] = b'F';
        res[11] = b'1';
        res[12] = b'F';
        res[13] = b'2';
        res[14] = b'F';
        res[15] = b'3';
        res[16] = b'\r';

        assert_eq!(
            serializer.to_bytes(
                SlcanCommand::Frame(CanFrame { 
                    id: Id::Extended(ExtendedId::new(0x12345678).unwrap()),
                    data: [0xf1, 0xf2, 0xf3, 0x00, 0x00, 0x00, 0x00, 0x00],
                    dlc: 3,
                    is_remote: true
                })
            ).unwrap(),
            res
        );
    }
}