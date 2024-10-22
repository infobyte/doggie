#![no_std]

fn hex_char_to_u8(hex_char: u8) -> Option<u8> {
    match hex_char {
        b'0'..=b'9' => Some(hex_char - b'0'), // Convert '0'-'9' to 0-9
        b'A'..=b'F' => Some(hex_char - b'A' + 10), // Convert 'A'-'F' to 10-15
        b'a'..=b'f' => Some(hex_char - b'a' + 10), // Convert 'a'-'f' to 10-15
        _ => None, // Handle invalid input
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StandardId(u16);

impl StandardId {
    pub const fn new(id: u16) -> Option<Self> {
        if id <= 0x7FF {
            Some(Self(id))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExtendedId(u32);

impl ExtendedId {
    pub const fn new(id: u32) -> Option<Self> {
        if id <= 0x1FFF_FFFF {
            Some(Self(id))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StandardFrame {
    id: StandardId,
    rtr: bool,
    dlc: usize,
    data: [u8; 8]
}

impl StandardFrame {
    pub fn new_t(id: u16, data: [u8; 8]) -> Option<Self> {
        let opt: Option<StandardId> = StandardId::new(id);
        match opt {
            Some(id) => {
                Some(Self {
                    id: id,
                    rtr: false,
                    dlc: data.iter().take_while(|&&b| b != 0).count(),
                    data: data
                })
            },
            None => None
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExtendedFrame {
    id: ExtendedId,
    rtr: bool,
    dlc: usize,
    data: [u8; 8]
}

impl ExtendedFrame {
    pub fn new_t(id: u32, data: [u8; 8]) -> Option<Self> {
        let opt = ExtendedId::new(id);
        match opt {
            Some(id) => {
                Some(Self {
                    id: id,
                    rtr: false,
                    dlc: data.iter().take_while(|&&b| b != 0).count(),
                    data: data
                })
            },
            None => None
        }
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
    SendStandardFrame(StandardFrame),           // t/r
    SendExtendedFrame(ExtendedFrame),           // T/R
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

    pub fn parse_byte(&mut self, byte: u8) -> Result<SlcanCommand, SlcanError> {
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
            b'O' => {
                if self.msg_len == 2 {
                    Ok(SlcanCommand::OpenChannel)
                } else {
                    Err(SlcanError::InvalidCommand)
                }
            },
            b'C' => {
                if self.msg_len == 2 {
                    Ok(SlcanCommand::CloseChannel)
                } else {
                    Err(SlcanError::InvalidCommand)
                }
            },
            b'F' => {
                if self.msg_len == 2 {
                    Ok(SlcanCommand::ReadStatusFlags)
                } else {
                    Err(SlcanError::InvalidCommand)
                }
            },
            b'L' => {
                if self.msg_len == 2 {
                    Ok(SlcanCommand::Listen)
                } else {
                    Err(SlcanError::InvalidCommand)
                }
            },
            b'S' => {
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
            },
            b't' => {
                if self.msg_len >= 6 {
                    let opt_id = StandardId::new((hex_char_to_u8(self.msg_buffer[1]).expect("Invalid hex") as u16) << 8 | (hex_char_to_u8(self.msg_buffer[2]).expect("Invalid hex") as u16) << 4 | hex_char_to_u8(self.msg_buffer[3]).expect("Invalid hex") as u16);
                    match opt_id {
                        Some(id) => {
                            let dlc = hex_char_to_u8(self.msg_buffer[4]).expect("Invalid hex") as usize;
                            if self.msg_len == dlc * 2 + 6 {
                                let mut data = [0; 8];

                                for i in 0..dlc  {
                                    data[i] = hex_char_to_u8(self.msg_buffer[5 + 2 * i]).expect("Invalid hex") << 4 | hex_char_to_u8(self.msg_buffer[6 + 2 * i]).expect("Invalid hex");
                                }

                                Ok(SlcanCommand::SendStandardFrame(
                                    StandardFrame {
                                        id: id,
                                        rtr: false,
                                        dlc: dlc,
                                        data: data
                                    }
                                ))
                            } else {
                                Err(SlcanError::InvalidCommand)
                            }
                        }
                        None => Err(SlcanError::InvalidCommand)
                    }
                } else {
                    Err(SlcanError::InvalidCommand)
                }
            },
            _ => Err(SlcanError::CommandNotImplemented)
        }
     }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_channel_valid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'O'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::OpenChannel));
    }

    #[test]
    fn test_open_channel_invalid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'O'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_close_channel_valid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'C'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::CloseChannel));
    }

    #[test]
    fn test_close_channel_invalid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'C'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_undefined_command() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::CommandNotImplemented));
    }

    #[test]
    fn test_too_long_command() {
        let mut parser = Slcan::new();

        for _ in 0..31 {
            assert_eq!(parser.parse_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        }
        assert_eq!(parser.parse_byte(b'X'), Err(SlcanError::MessageTooLong));
    }

    #[test]
    fn test_status_flag_valid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'F'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::ReadStatusFlags));
    }

    #[test]
    fn test_status_flag_invalid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'F'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_listen_valid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'L'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::Listen));
    }

    #[test]
    fn test_listen_invalid() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'L'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'X'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_set_bitrate_0() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'0'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN10KB)));
    }

    #[test]
    fn test_set_bitrate_1() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN20KB)));
    }

    #[test]
    fn test_set_bitrate_2() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN50KB)));
    }

    #[test]
    fn test_set_bitrate_3() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN100KB)));
    }

    #[test]
    fn test_set_bitrate_4() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'4'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN125KB)));
    }

    #[test]
    fn test_set_bitrate_5() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'5'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN250KB)));
    }

    #[test]
    fn test_set_bitrate_6() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'6'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN500KB)));
    }

    #[test]
    fn test_set_bitrate_7() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'7'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN800KB)));
    }

    #[test]
    fn test_set_bitrate_8() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'8'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SetBitrate(SlcanBitrates::CAN1000KB)));
    }

    #[test]
    fn test_set_bitrate_invalid_bitrate() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'9'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_set_bitrate_invalid_command() {
        let mut parser = Slcan::new();

        assert_eq!(parser.parse_byte(b'S'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }

    #[test]
    fn test_standard_frame_t_len_0_valid_data() {
        let mut parser = Slcan::new();
        // t1230 : can_id 0x123, can_dlc 0, no data
        assert_eq!(parser.parse_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'0'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SendStandardFrame(
            StandardFrame { 
                id: StandardId(0x123), 
                rtr: false, 
                dlc: 0, 
                data: [0; 8] }
        )));
    }

    #[test]
    fn test_standard_frame_t_len_0_invalid_data() {
        let mut parser = Slcan::new();
        // t1230 : can_id 0x123, can_dlc 0, data 0x11
        assert_eq!(parser.parse_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'0'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::InvalidCommand))
    }

    #[test]
    fn test_standard_frame_t_len_3_valid_data() {
        let mut parser = Slcan::new();
        // t4563112233 : can_id 0x456, can_dlc 3, data 0x11 0x22 0x33
        assert_eq!(parser.parse_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'4'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'5'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'6'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'2'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Ok(SlcanCommand::SendStandardFrame(
            StandardFrame { 
                id: StandardId(0x456), 
                rtr: false, 
                dlc: 3, 
                data: [0x11, 0x22, 0x33, 0, 0 ,0 , 0, 0] }
        )));
    }

    #[test]
    fn test_standard_frame_t_len_3_invalid_data() {
        let mut parser = Slcan::new();
        // t4563112233 : can_id 0x456, can_dlc 3, data 0x11
        assert_eq!(parser.parse_byte(b't'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'4'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'5'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'6'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'3'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'1'), Ok(SlcanCommand::IncompleteMessage));
        assert_eq!(parser.parse_byte(b'\r'), Err(SlcanError::InvalidCommand));
    }
}