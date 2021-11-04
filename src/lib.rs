#![no_std]

use embedded_time::duration::Milliseconds;
use commands::{AtExecute, AtRead, AtWrite};

pub mod commands;
pub mod tcp_client;

#[derive(Debug)]
pub enum Error<S: core::fmt::Debug> {
    DecodingFailed,
    SerialError(S),
    Timeout,
    BufferOverflow,
    ConnectFailed
}

impl<S: core::fmt::Debug> From<S> for Error<S> {
    fn from(value: S) -> Self {
        Error::SerialError(value)
    }
}

pub trait Serial {
    type SerialError: core::fmt::Debug;
}
pub trait SerialReadTimeout: Serial {
    fn read_exact(&mut self, buf: &mut [u8], timeout: Milliseconds) -> Result<Option<()>, Self::SerialError>;

    fn read_line<'a>(
        &mut self,
        out: &'a mut [u8],
        timeout: Milliseconds,
    ) -> Result<Option<&'a str>, Self::SerialError>;
}

pub trait SerialWrite: Serial {
    fn write(&mut self, buf: &[u8]) -> Result<(), Self::SerialError>;
}

pub trait AtModem: SerialWrite + SerialReadTimeout {
    fn read<C: AtRead>(&mut self, command: C, timeout: Milliseconds) -> Result<C::Output, Error<Self::SerialError>>;

    fn write<'a, C: AtWrite<'a>>(
        &mut self,
        command: C,
        param: C::Input,
        timeout: Milliseconds,
    ) -> Result<C::Output, Error<Self::SerialError>>;

    fn execute<C: AtExecute>(
        &mut self,
        command: C,
        timeout: Milliseconds,
    ) -> Result<C::Output, Error<Self::SerialError>>;
}

fn drain_relay<T>(relay: &mut T, mut timeout: Milliseconds) -> Result<bool, T::SerialError>
where
    T: SerialReadTimeout,
{
    let mut recv_buf = [0u8; 1];
    let mut drained_data = false;
    loop {
        let res = relay.read_exact(&mut recv_buf, timeout);
        match res {
            Err(error) => return Err(error),
            Ok(None) => return Ok(drained_data),
            _ => {
                log::trace!("DRAIN: {:?}", core::str::from_utf8(&recv_buf));
                drained_data = true;
            }
        };

        // All subsequent loops should have a timeout of 0 so they never block
        timeout = Milliseconds(0);
    }
}