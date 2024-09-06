use crate::{Utf8Duplexer, Utf8Reader};
use duplex::Duplex;
use std::cmp::min;
use std::io::{self, copy, repeat, Cursor, Read, Write};
use std::str;
#[cfg(feature = "layered-io")]
use {
    layered_io::{HalfDuplexLayered, ReadLayered, Status},
    std::cmp::max,
};

pub(crate) trait Utf8ReaderInternals<Inner: Read>: Read {
    fn impl_(&mut self) -> &mut Utf8Input;
    #[cfg(feature = "layered-io")]
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
    #[cfg(feature = "layered-io")]
    fn into_inner(self) -> Inner;
}

#[cfg(feature = "layered-io")]
pub(crate) trait Utf8ReaderInternalsLayered<Inner: ReadLayered>:
    Utf8ReaderInternals<Inner> + ReadLayered
{
}

impl<Inner: Read> Utf8ReaderInternals<Inner> for Utf8Reader<Inner> {
    fn impl_(&mut self) -> &mut Utf8Input {
        &mut self.input
    }

    #[cfg(feature = "layered-io")]
    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    #[cfg(feature = "layered-io")]
    fn into_inner(self) -> Inner {
        self.inner
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: ReadLayered> Utf8ReaderInternalsLayered<Inner> for Utf8Reader<Inner> {}

impl<Inner: Duplex + Read + Write> Utf8ReaderInternals<Inner> for Utf8Duplexer<Inner> {
    fn impl_(&mut self) -> &mut Utf8Input {
        &mut self.input
    }

    #[cfg(feature = "layered-io")]
    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    #[cfg(feature = "layered-io")]
    fn into_inner(self) -> Inner {
        self.inner
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: HalfDuplexLayered> Utf8ReaderInternalsLayered<Inner> for Utf8Duplexer<Inner> {}

pub(crate) struct Utf8Input {
    /// A queue of bytes which have not been read but which have not been
    /// translated into the output yet.
    overflow: Vec<u8>,
}

impl Utf8Input {
    /// Construct a new instance of `Utf8Input`.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            overflow: Vec::new(),
        }
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[inline]
    pub(crate) fn read_str<Inner: Read>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<usize> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_exact` but produces the result in a `str`.
    #[inline]
    pub(crate) fn read_exact_str<Inner: Read>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<()> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read_exact(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn read_str_with_status<Inner: ReadLayered>(
        internals: &mut impl Utf8ReaderInternalsLayered<Inner>,
        buf: &mut str,
    ) -> io::Result<(usize, Status)> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        let (size, status) = internals.read_with_status(unsafe { buf.as_bytes_mut() })?;

        debug_assert!(buf.is_char_boundary(size));

        Ok((size, status))
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn read_exact_str_using_status<Inner: ReadLayered>(
        internals: &mut impl Utf8ReaderInternalsLayered<Inner>,
        buf: &mut str,
    ) -> io::Result<Status> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read_exact_using_status(unsafe { buf.as_bytes_mut() })
    }

    #[cfg(feature = "layered-io")]
    pub(crate) fn read_with_status<Inner: ReadLayered>(
        internals: &mut impl Utf8ReaderInternalsLayered<Inner>,
        buf: &mut [u8],
    ) -> io::Result<(usize, Status)> {
        let (nread, done) = Self::process_old_data(internals, buf)?;
        if done {
            return Ok((nread, Status::active()));
        }

        let (size, status) = internals.inner_mut().read_with_status(&mut buf[nread..])?;

        let (nread, done) = Self::process_new_data(internals, buf, nread, size, status.is_end())?;
        Ok((nread, if done { status } else { Status::active() }))
    }

    fn process_old_data<Inner: Read>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<(usize, bool)> {
        // To ensure we can always make progress, callers should always use a
        // buffer of at least 4 bytes.
        if buf.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer for reading from Utf8Reader must be at least 4 bytes long",
            ));
        }

        let mut nread = 0;

        if !internals.impl_().overflow.is_empty() {
            nread += internals
                .impl_()
                .process_overflow(&mut buf[nread..], IncompleteHow::Include)
                .unwrap();
            if !internals.impl_().overflow.is_empty() {
                return Ok((nread, true));
            }
        }

        Ok((nread, false))
    }

    pub(crate) fn process_new_data<Inner: Read>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut [u8],
        mut nread: usize,
        size: usize,
        is_end: bool,
    ) -> io::Result<(usize, bool)> {
        nread += size;

        // We may have overwritten part of a codepoint; overwrite the rest of
        // the buffer.
        // TODO: Use [`fill`] when it becomes available:
        // https://doc.rust-lang.org/std/primitive.slice.html#method.fill
        copy(
            &mut repeat(b'\0').take((buf.len() - nread) as u64),
            &mut Cursor::new(&mut buf[nread..]),
        )
        .unwrap();

        match str::from_utf8(&buf[..nread]) {
            Ok(_) => Ok((nread, true)),
            Err(error) => {
                let (valid, after_valid) = buf[..nread].split_at(error.valid_up_to());
                nread = valid.len();

                assert!(internals.impl_().overflow.is_empty());
                internals.impl_().overflow.extend_from_slice(after_valid);

                let incomplete_how = if is_end {
                    IncompleteHow::Replace
                } else {
                    IncompleteHow::Exclude
                };
                nread += internals
                    .impl_()
                    .process_overflow(&mut buf[nread..], incomplete_how)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid UTF-8"))?;
                Ok((nread, internals.impl_().overflow.is_empty()))
            }
        }
    }

    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn minimum_buffer_size<Inner: ReadLayered>(
        internals: &impl Utf8ReaderInternals<Inner>,
    ) -> usize {
        // UTF-8 needs at most 4 bytes per codepoint.
        max(4, internals.inner().minimum_buffer_size())
    }

    /// If normal reading encounters invalid bytes, the data is copied into
    /// `internals.impl_().overflow` as it may need to expand to make room for
    /// the U+FFFD's, and we may need to hold on to some of it until the next
    /// `read` call.
    ///
    /// TODO: This code could be significantly optimized.
    #[cold]
    fn process_overflow(&mut self, buf: &mut [u8], incomplete_how: IncompleteHow) -> Option<usize> {
        let mut nread = 0;

        loop {
            let num = min(buf[nread..].len(), self.overflow.len());
            match str::from_utf8(&self.overflow[..num]) {
                Ok(_) => {
                    buf[nread..nread + num].copy_from_slice(&self.overflow[..num]);
                    self.overflow.copy_within(num.., 0);
                    self.overflow.resize(self.overflow.len() - num, 0);
                    nread += num;
                }
                Err(error) => {
                    let (valid, after_valid) = self.overflow[..num].split_at(error.valid_up_to());
                    let valid_len = valid.len();
                    let after_valid_len = after_valid.len();
                    buf[nread..nread + valid_len].copy_from_slice(valid);
                    self.overflow.copy_within(valid_len.., 0);
                    self.overflow.resize(self.overflow.len() - valid_len, 0);
                    nread += valid_len;

                    if let Some(invalid_sequence_length) = error.error_len() {
                        if '\u{fffd}'.len_utf8() <= buf[nread..].len() {
                            nread += '\u{fffd}'.encode_utf8(&mut buf[nread..]).len();
                            self.overflow.copy_within(invalid_sequence_length.., 0);
                            self.overflow
                                .resize(self.overflow.len() - invalid_sequence_length, 0);
                            continue;
                        }
                    } else {
                        match incomplete_how {
                            IncompleteHow::Replace => {
                                if '\u{fffd}'.len_utf8() <= buf[nread..].len() {
                                    nread += '\u{fffd}'.encode_utf8(&mut buf[nread..]).len();
                                    self.overflow.clear();
                                } else if self.overflow.is_empty() {
                                    return None;
                                }
                            }
                            IncompleteHow::Include if after_valid_len == self.overflow.len() => {
                                if !buf[nread..].is_empty() {
                                    let num = min(buf[nread..].len(), after_valid_len);
                                    buf[nread..nread + num].copy_from_slice(&self.overflow[..num]);
                                    nread += num;
                                    self.overflow.copy_within(num.., 0);
                                    self.overflow.resize(self.overflow.len() - num, 0);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            break;
        }

        Some(nread)
    }

    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn abandon<Inner: ReadLayered>(internals: &mut impl Utf8ReaderInternals<Inner>) {
        internals.impl_().overflow.clear();
        internals.inner_mut().abandon()
    }

    #[cfg(feature = "layered-io")]
    #[inline]
    pub(crate) fn suggested_buffer_size<Inner: ReadLayered>(
        internals: &impl Utf8ReaderInternals<Inner>,
    ) -> usize {
        max(
            Self::minimum_buffer_size(internals),
            internals.inner().suggested_buffer_size(),
        )
    }

    #[inline]
    pub(crate) fn read<Inner: Read>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        let (nread, done) = Self::process_old_data(internals, buf)?;
        if done {
            return Ok(nread);
        }

        let (size, is_end) = match internals.inner_mut().read(&mut buf[nread..]) {
            Ok(0) => (0, true),
            Ok(size) => (size, false),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => (0, false),
            Err(err) => return Err(err),
        };

        let (nread, done) = Self::process_new_data(internals, buf, nread, size, is_end)?;

        match (nread, done) {
            (0, true) => Ok(0),
            (0, false) => Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "read zero bytes from stream",
            )),
            (_, _) => Ok(nread),
        }
    }

    #[inline]
    pub(crate) fn read_to_string<Inner: Read>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut String,
    ) -> io::Result<usize> {
        // Safety: Our `read` implementation already ensures that its output
        // is UTF-8, so we can just unwrap it here.
        internals.read_to_end(unsafe { buf.as_mut_vec() })
    }
}

/// What to do when there is an incomplete UTF-8 sequence at the end of
/// the overflow buffer.
enum IncompleteHow {
    /// Include the incomplete sequence in the output.
    Include,
    /// Leave the incomplete sequence in the overflow buffer.
    Exclude,
    /// Replace the incomplete sequence with U+FFFD.
    Replace,
}
