// The longest serial can msg length is 34, plus 2 for \r\n
pub const MAX_CMD_LEN: usize = 36;

// Max length of the serial pipe
pub const PIPE_SIZE: usize = MAX_CMD_LEN * 10;
