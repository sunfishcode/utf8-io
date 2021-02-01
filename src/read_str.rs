#[cfg(feature = "layered-io")]
use layered_io::{ReadLayered, Status};
use std::io::{self, Read};

/// Extend the `Read` trait with `read_str`, a method for reading UTF-8 data.
pub trait ReadStr: Read {
    /// Like `read` but produces the result in a `str`. Be sure to check the
    /// `size` field of the return value to see how many bytes were written.
    ///
    /// `buf` must be at least 4 bytes long, so that any valid UTF-8 codepoint
    /// can be read.
    fn read_str(&mut self, buf: &mut str) -> io::Result<usize>;

    /// Like `read_exact` but produces the result in a `str`.
    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        default_read_exact_str(self, buf)
    }
}

/// Extend the `ReadLayered` trait with `read_str_with_status`, a method for
/// reading UTF-8 data.
#[cfg(feature = "layered-io")]
pub trait ReadStrLayered: ReadLayered + ReadStr {
    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the return value to see how many bytes were written.
    ///
    /// `buf` must be at least 4 bytes long, so that any valid UTF-8 codepoint
    /// can be read.
    fn read_str_with_status(&mut self, buf: &mut str) -> io::Result<(usize, Status)>;

    /// Like `read_exact` but produces the result in a `str`.
    ///
    /// Also, like `ReadStr::read_exact_str`, but uses `read_str_with_status`
    /// to avoid performing an extra `read` at the end.
    #[inline]
    fn read_exact_str_using_status(&mut self, buf: &mut str) -> io::Result<Status> {
        default_read_exact_str_using_status(self, buf)
    }
}

/// Default implementation of [`ReadStr::read_exact_str`].
pub fn default_read_exact_str<Inner: ReadStr + ?Sized>(
    inner: &mut Inner,
    mut buf: &mut str,
) -> io::Result<()> {
    while !buf.is_empty() {
        match inner.read_str(buf) {
            Ok(0) => break,
            Ok(size) => buf = buf.split_at_mut(size).1,
            Err(e) => return Err(e),
        }
    }

    if buf.is_empty() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "failed to fill whole buffer",
        ))
    }
}

/// Default implementation of [`ReadStrLayered::read_exact_str_using_status`].
#[cfg(feature = "layered-io")]
pub fn default_read_exact_str_using_status<Inner: ReadStrLayered + ?Sized>(
    inner: &mut Inner,
    mut buf: &mut str,
) -> io::Result<Status> {
    let mut result_status = Status::active();

    while !buf.is_empty() {
        match inner.read_str_with_status(buf) {
            Ok((size, status)) => {
                buf = buf.split_at_mut(size).1;
                if status.is_end() {
                    result_status = status;
                    break;
                }
            }
            Err(e) => return Err(e),
        }
    }

    if buf.is_empty() {
        Ok(result_status)
    } else {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "failed to fill whole buffer",
        ))
    }
}
