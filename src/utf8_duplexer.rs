use crate::{utf8_input::Utf8Input, utf8_output::Utf8Output, ReadStr, WriteStr};
use duplex::{Duplex, HalfDuplex};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, RawFd};
use std::{
    fmt,
    io::{self, Read, Write},
    str,
};
#[cfg(feature = "terminal-io")]
use terminal_io::{DuplexTerminal, ReadTerminal, Terminal, TerminalColorSupport, WriteTerminal};
#[cfg(windows)]
use unsafe_io::{AsRawHandleOrSocket, RawHandleOrSocket};
#[cfg(feature = "layered-io")]
use {
    crate::ReadStrLayered,
    layered_io::{Bufferable, HalfDuplexLayered, ReadLayered, Status, WriteLayered},
    std::cmp::max,
};

/// An interactive UTF-8 stream, combining `Utf8Reader` and `Utf8Writer`.
pub struct Utf8Duplexer<Inner: HalfDuplex> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// UTF-8 translation state.
    pub(crate) input: Utf8Input,
    pub(crate) output: Utf8Output,
}

impl<Inner: HalfDuplex> Utf8Duplexer<Inner> {
    /// Construct a new instance of `Utf8Duplexer` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            input: Utf8Input::new(),
            output: Utf8Output::new(),
        }
    }

    /// Flush any pending output and return the inner output stream.
    #[inline]
    pub fn into_inner(self) -> io::Result<Inner> {
        Utf8Output::into_inner(self)
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: HalfDuplexLayered> Utf8Duplexer<Inner> {
    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn close_into_inner(self) -> io::Result<Inner> {
        Utf8Output::into_inner(self)
    }

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn abandon_into_inner(self) -> Inner {
        Utf8Output::abandon_into_inner(self)
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: Duplex + DuplexTerminal> Terminal for Utf8Duplexer<Inner> {}

#[cfg(feature = "terminal-io")]
impl<Inner: Duplex + DuplexTerminal> ReadTerminal for Utf8Duplexer<Inner> {
    #[inline]
    fn is_line_by_line(&self) -> bool {
        self.inner.is_line_by_line()
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        self.inner.is_input_terminal()
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: Duplex + DuplexTerminal> WriteTerminal for Utf8Duplexer<Inner> {
    #[inline]
    fn color_support(&self) -> TerminalColorSupport {
        self.inner.color_support()
    }

    #[inline]
    fn color_preference(&self) -> bool {
        self.inner.color_preference()
    }

    #[inline]
    fn is_output_terminal(&self) -> bool {
        self.inner.is_output_terminal()
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: HalfDuplex + DuplexTerminal> DuplexTerminal for Utf8Duplexer<Inner> {}

impl<Inner: HalfDuplex> ReadStr for Utf8Duplexer<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<usize> {
        Utf8Input::read_str(self, buf)
    }

    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        Utf8Input::read_exact_str(self, buf)
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: HalfDuplexLayered> ReadStrLayered for Utf8Duplexer<Inner> {
    #[inline]
    fn read_str_with_status(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        Utf8Input::read_str_with_status(self, buf)
    }

    #[inline]
    fn read_exact_str_using_status(&mut self, buf: &mut str) -> io::Result<Status> {
        Utf8Input::read_exact_str_using_status(self, buf)
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: HalfDuplexLayered> ReadLayered for Utf8Duplexer<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        Utf8Input::read_with_status(self, buf)
    }

    #[inline]
    fn minimum_buffer_size(&self) -> usize {
        Utf8Input::minimum_buffer_size(self)
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: HalfDuplexLayered> Bufferable for Utf8Duplexer<Inner> {
    #[inline]
    fn abandon(&mut self) {
        Utf8Input::abandon(self);
        Utf8Output::abandon(self);
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        max(
            Utf8Input::suggested_buffer_size(self),
            Utf8Output::suggested_buffer_size(self),
        )
    }
}

impl<Inner: HalfDuplex> Read for Utf8Duplexer<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Utf8Input::read(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        Utf8Input::read_to_string(self, buf)
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: HalfDuplexLayered> WriteLayered for Utf8Duplexer<Inner> {
    #[inline]
    fn close(&mut self) -> io::Result<()> {
        Utf8Output::close(self)
    }
}

impl<Inner: HalfDuplex> WriteStr for Utf8Duplexer<Inner> {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        Utf8Output::write_str(self, s)
    }
}

impl<Inner: HalfDuplex> Duplex for Utf8Duplexer<Inner> {}

impl<Inner: HalfDuplex> Write for Utf8Duplexer<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Utf8Output::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Utf8Output::flush(self)
    }
}

#[cfg(not(windows))]
impl<Inner: HalfDuplex + AsRawFd> AsRawFd for Utf8Duplexer<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: HalfDuplex + AsRawHandleOrSocket> AsRawHandleOrSocket for Utf8Duplexer<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

impl<Inner: HalfDuplex + fmt::Debug> fmt::Debug for Utf8Duplexer<Inner> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut b = f.debug_struct("Utf8Duplexer");
        b.field("inner", &self.inner);
        b.finish()
    }
}
