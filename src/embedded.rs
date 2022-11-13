//! Opt-in support for embedded-hal traits.
//!
//! Can be enabled with the "embedded" cargo feature.

use std::io;

use embedded_hal::serial::{ErrorType, ErrorKind};

use crate::SerialPort;

#[derive(Debug, Copy, Clone)]
pub struct SerialError {
    kind: io::ErrorKind,
}

impl embedded_hal::serial::Error for SerialError {
    fn kind(&self) -> ErrorKind {
        #[allow(clippy::match_single_binding)]
        match self.kind {
            _ => ErrorKind::Other,
        }
    }
}

impl From<io::Error> for SerialError {
    fn from(e: io::Error) -> Self {
        SerialError {
            kind: e.kind(),
        }
    }
}

impl ErrorType for Box<dyn SerialPort> {
    type Error = SerialError;
}


mod nonblocking {
    use super::*;
    use embedded_hal_nb::serial;

    fn io_error_to_nb(err: io::Error) -> nb::Error<SerialError> {
        match err.kind() {
            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => nb::Error::WouldBlock,
            other => nb::Error::Other(SerialError { kind: other }),
        }
    }

    impl serial::Read<u8> for Box<dyn SerialPort> {
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            let mut buffer = [0; 1];
            let bytes_read = io::Read::read(self, &mut buffer).map_err(io_error_to_nb)?;
            if bytes_read > 0 {
                Ok(buffer[0])
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    impl serial::Write<u8> for Box<dyn SerialPort> {
        fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
            io::Write::write(self, &[word])
                .map_err(io_error_to_nb)
                .map(|_| ())
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            io::Write::flush(self).map_err(io_error_to_nb)
        }
    }
}

mod blocking {
    use super::*;
    use embedded_hal::serial;

    impl serial::Write<u8> for Box<dyn SerialPort> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            Ok(io::Write::write_all(self, buffer)?)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(io::Write::flush(self)?)
        }
    }
}