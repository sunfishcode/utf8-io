use crate::{default_write_str, Utf8Duplexer, Utf8Writer};
use duplex::Duplex;
#[cfg(feature = "layered-io")]
use layered_io::{HalfDuplexLayered, WriteLayered};
use std::cmp::min;
use std::io::{self, Read, Write};
use std::str;

pub(crate) trait Utf8WriterInternals<Inner: Write>: Write {
    fn impl_(&mut self) -> &mut Utf8Output;
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
    fn into_inner(self) -> Inner;
    fn write_incomplete(&mut self, utf8_len: usize) -> io::Result<()>;
}

#[cfg(feature = "layered-io")]
pub(crate) trait Utf8WriterInternalsLayered<Inner: WriteLayered>:
    Utf8WriterInternals<Inner> + WriteLayered
{
}

impl<Inner: Write> Utf8WriterInternals<Inner> for Utf8Writer<Inner> {
    fn impl_(&mut self) -> &mut Utf8Output {
        &mut self.output
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }

    fn write_incomplete(&mut self, utf8_len: usize) -> io::Result<()> {
        let to_write = &self.output.incomplete[..utf8_len];
        self.output.incomplete_len = 0;
        self.inner.write_all(to_write)?;
        Ok(())
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: WriteLayered> Utf8WriterInternalsLayered<Inner> for Utf8Writer<Inner> {}

impl<Inner: Duplex + Read + Write> Utf8WriterInternals<Inner> for Utf8Duplexer<Inner> {
    fn impl_(&mut self) -> &mut Utf8Output {
        &mut self.output
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }

    fn write_incomplete(&mut self, utf8_len: usize) -> io::Result<()> {
        let to_write = &self.output.incomplete[..utf8_len];
        self.output.incomplete_len = 0;
        self.inner.write_all(to_write)?;
        Ok(())
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: HalfDuplexLayered> Utf8WriterInternalsLayered<Inner> for Utf8Duplexer<Inner> {}

pub(crate) struct Utf8Output {
    incomplete: [u8; 4],
    incomplete_len: u8,
}

impl Utf8Output {
    /// Construct a new instance of `Utf8Output`.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            incomplete: [0, 0, 0, 0],
            incomplete_len: 0,
        }
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub(crate) fn into_inner<Inner: Write>(
        mut internals: impl Utf8WriterInternals<Inner>,
    ) -> io::Result<Inner> {
        internals.flush()?;
        Ok(internals.into_inner())
    }

    /// Return the underlying stream object.
    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn abandon_into_inner<Inner: Write>(
        internals: impl Utf8WriterInternals<Inner>,
    ) -> Inner {
        internals.into_inner()
    }

    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn close<Inner: WriteLayered>(
        internals: &mut impl Utf8WriterInternalsLayered<Inner>,
    ) -> io::Result<()> {
        internals.inner_mut().close()
    }

    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn abandon<Inner: WriteLayered>(internals: &mut impl Utf8WriterInternals<Inner>) {
        internals.inner_mut().abandon()
    }

    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn suggested_buffer_size<Inner: WriteLayered>(
        internals: &impl Utf8WriterInternals<Inner>,
    ) -> usize {
        internals.inner().suggested_buffer_size()
    }

    #[inline]
    pub(crate) fn write_str<Inner: Write>(
        internals: &mut impl Utf8WriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        default_write_str(internals.inner_mut(), s)
    }

    pub(crate) fn write<Inner: Write>(
        internals: &mut impl Utf8WriterInternals<Inner>,
        mut buf: &[u8],
    ) -> io::Result<usize> {
        let mut written = 0;

        // If we have incomplete bytes from the previous `write`, try to
        // complete them.
        let incomplete_len = usize::from(internals.impl_().incomplete_len);
        let mut buf_len = buf.len();
        if incomplete_len != 0 {
            // Compute how any bytes we need for the UTF-8 encoding.
            let utf8_len = match internals.impl_().incomplete[0] & 0x30 {
                0x20 => 3,
                0x30 => 4,
                _ => 2,
            };

            // We're only given so many bytes.
            let copy_len = min(utf8_len - incomplete_len, buf_len);

            // Copy `copy_len` bytes from `buf` into the `incomplete` buffer.
            internals.impl_().incomplete[incomplete_len..(incomplete_len + copy_len)]
                .copy_from_slice(&buf[..copy_len]);
            written += copy_len;

            let new_incomplete_len = incomplete_len + copy_len;
            internals.impl_().incomplete_len = new_incomplete_len as u8;

            // If the sequence is still incomplete, wait for the next `write`.
            if new_incomplete_len < utf8_len {
                return Ok(written);
            }

            // The sequence is complete; write it.
            internals.write_incomplete(utf8_len)?;
            buf = &buf[copy_len..];
            buf_len = buf.len();
        }

        // If the buffer is UTF-8, write it. If it has incomplete bytes at the
        // end, write what we can and save the incomplete bytes for the next
        // `write`. If it's invalid, write what we can and fail.
        match str::from_utf8(buf) {
            Ok(s) => Self::write_str(internals, s).map(|()| written + buf_len),
            Err(error) => {
                let valid_up_to = error.valid_up_to();
                if valid_up_to != 0 {
                    internals
                        .inner_mut()
                        .write_all(&buf[..valid_up_to])
                        .map(|()| valid_up_to)?;
                }
                if error.error_len().is_none() {
                    let incomplete_len = buf_len - valid_up_to;
                    internals.impl_().incomplete[..incomplete_len]
                        .copy_from_slice(&buf[valid_up_to..]);
                    internals.impl_().incomplete_len = incomplete_len as u8;
                    Ok(written + buf_len)
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidData, error))
                }
            }
        }
    }

    #[inline]
    pub(crate) fn flush<Inner: Write>(
        internals: &mut impl Utf8WriterInternals<Inner>,
    ) -> io::Result<()> {
        if internals.impl_().incomplete_len != 0 {
            internals.impl_().incomplete_len = 0;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "incomplete UTF-8 encoding at flush",
            ));
        }
        internals.inner_mut().flush()
    }
}

impl Drop for Utf8Output {
    fn drop(&mut self) {
        if self.incomplete_len == 0 {
            // oll korrect
        } else {
            panic!("output text stream not ended on UTF-8 boundary");
        }
    }
}
