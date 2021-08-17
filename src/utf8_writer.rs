use crate::utf8_output::Utf8Output;
use crate::WriteStr;
#[cfg(feature = "layered-io")]
use layered_io::{Bufferable, WriteLayered};
use std::io::{self, Write};
use std::{fmt, str};
#[cfg(feature = "terminal-io")]
use terminal_io::{Terminal, TerminalColorSupport, WriteTerminal};
#[cfg(windows)]
use unsafe_io::os::windows::{
    AsHandleOrSocket, AsRawHandleOrSocket, BorrowedHandleOrSocket, RawHandleOrSocket,
};
#[cfg(not(windows))]
use {
    io_lifetimes::{AsFd, BorrowedFd},
    unsafe_io::os::rsix::{AsRawFd, RawFd},
};

/// A [`Write`] implementation which translates into an output `Write`
/// producing a valid UTF-8 sequence from an arbitrary byte sequence from an
/// arbitrary byte sequence. Attempts to write invalid encodings are reported
/// as errors.
///
/// This type's `write` is not guaranteed to perform a single underlying
/// `write` operation, because short writes could produce invalid UTF-8, so
/// `write` will retry as needed.
pub struct Utf8Writer<Inner: Write> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// UTF-8 translation state.
    pub(crate) output: Utf8Output,
}

impl<Inner: Write> Utf8Writer<Inner> {
    /// Construct a new instance of `Utf8Writer` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            output: Utf8Output::new(),
        }
    }

    /// Flush any pending output and return the inner stream.
    #[inline]
    pub fn into_inner(mut self) -> io::Result<Inner> {
        self.flush()?;
        Utf8Output::into_inner(self)
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: WriteLayered> Utf8Writer<Inner> {
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
impl<Inner: Write + WriteTerminal> Terminal for Utf8Writer<Inner> {}

#[cfg(feature = "terminal-io")]
impl<Inner: Write + WriteTerminal> WriteTerminal for Utf8Writer<Inner> {
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

#[cfg(feature = "layered-io")]
impl<Inner: WriteLayered> WriteLayered for Utf8Writer<Inner> {
    #[inline]
    fn close(&mut self) -> io::Result<()> {
        Utf8Output::close(self)
    }
}

impl<Inner: Write> WriteStr for Utf8Writer<Inner> {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        Utf8Output::write_str(self, s)
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: WriteLayered> Bufferable for Utf8Writer<Inner> {
    #[inline]
    fn abandon(&mut self) {
        Utf8Output::abandon(self)
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        Utf8Output::suggested_buffer_size(self)
    }
}

impl<Inner: Write> Write for Utf8Writer<Inner> {
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
impl<Inner: Write + AsRawFd> AsRawFd for Utf8Writer<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(not(windows))]
impl<Inner: Write + AsFd> AsFd for Utf8Writer<Inner> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

#[cfg(windows)]
impl<Inner: Write + AsRawHandleOrSocket> AsRawHandleOrSocket for Utf8Writer<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl<Inner: Write + AsHandleOrSocket> AsHandleOrSocket for Utf8Writer<Inner> {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.inner.as_handle_or_socket()
    }
}

impl<Inner: Write + fmt::Debug> fmt::Debug for Utf8Writer<Inner> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("Utf8Writer");
        b.field("inner", &self.inner);
        b.finish()
    }
}
