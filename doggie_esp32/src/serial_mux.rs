use embedded_io_async::{Read, Write, ErrorType, ErrorKind, Error};
use embassy_futures::select::{select, Either};
use defmt::info;

#[derive(Debug)]
pub struct SerialMuxError {}

impl<'d> Error for SerialMuxError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

#[derive(PartialEq)]
enum SerialMuxState {
    Init,
    First,
    Second,
}

pub struct SerialMux<S1, S2>
where
    S1: Write + Read,
    S2: Write + Read,
{
    s1: S1,
    s2: S2,
    state: SerialMuxState,
}

impl<S1, S2> SerialMux<S1, S2>
where
    S1: Write + Read,
    S2: Write + Read,
{
    pub fn new(s1: S1, s2: S2) -> Self {
        Self { s1, s2, state: SerialMuxState::Init }
    }

    async fn auto_select(&mut self, buf: &mut [u8]) {
        let mut buf1: [u8;1] = [0];
        let mut buf2: [u8;1] = [0];

        loop {
            match select(
                self.s1.read(&mut buf1),
                self.s2.read(&mut buf2),
            ).await {
                Either::First(Ok(_)) => {
                    info!("First serial selected");
                    buf[0] = buf1[0];
                    self.state = SerialMuxState::First;
                    return
                },
                Either::Second(Ok(_)) => {
                    info!("Second serial selected");
                    buf[0] = buf2[0];
                    self.state = SerialMuxState::Second;
                    return
                },
                _ => {}
            }
            
        }
    }
}


impl<S1, S2> ErrorType for SerialMux<S1,S2>
where
    S1: Write + Read,
    S2: Write + Read,
{
    type Error = SerialMuxError;
}

impl<S1, S2> Read for SerialMux<S1, S2>
where
    S1: Write + Read,
    S2: Write + Read,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.state == SerialMuxState::Init {
            self.auto_select(buf).await;
        }

        match self.state {
            SerialMuxState::First => {
                match self.s1.read(buf).await {
                    Err(_) => Err(SerialMuxError {}),
                    Ok(res) => Ok(res),
                }
            },
            SerialMuxState::Second => {
                match self.s2.read(buf).await {
                    Err(_) => Err(SerialMuxError {}),
                    Ok(res) => Ok(res),
                }
            },
            SerialMuxState::Init => {
                Err(SerialMuxError {})
            }
        }    }
}

impl<S1, S2> Write for SerialMux<S1, S2>
where
    S1: Write + Read,
    S2: Write + Read,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if self.state == SerialMuxState::Init {
            self.auto_select(&mut [0]).await;
        }
        
        match self.state {
            SerialMuxState::First => {
                match self.s1.write(buf).await {
                    Err(_) => Err(SerialMuxError {}),
                    Ok(res) => Ok(res),
                }
            },
            SerialMuxState::Second => {
                match self.s2.write(buf).await {
                    Err(_) => Err(SerialMuxError {}),
                    Ok(res) => Ok(res),
                }
            },
            SerialMuxState::Init => {
                Err(SerialMuxError {})
            }
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

