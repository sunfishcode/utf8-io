use crate::{default_write_str, Utf8Duplexer, Utf8Writer};
use duplex::Duplex;
#[cfg(feature = "layered-io")]
use layered_io::{HalfDuplexLayered, WriteLayered};
use std::{
    io::{self, Read, Write},
    str,
};

pub(crate) trait Utf8WriterInternals<Inner: Write>: Write {
    fn impl_(&mut self) -> &mut Utf8Output;
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
    fn into_inner(self) -> Inner;
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
}

#[cfg(feature = "layered-io")]
impl<Inner: HalfDuplexLayered> Utf8WriterInternalsLayered<Inner> for Utf8Duplexer<Inner> {}

pub(crate) struct Utf8Output {}

impl Utf8Output {
    /// Construct a new instance of `Utf8Output`.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {}
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
        buf: &[u8],
    ) -> io::Result<usize> {
        match str::from_utf8(buf) {
            Ok(s) => Self::write_str(internals, s).map(|_| buf.len()),
            Err(error) if error.valid_up_to() != 0 => internals
                .inner_mut()
                .write_all(&buf[..error.valid_up_to()])
                .map(|_| error.valid_up_to()),
            Err(error) => Err(io::Error::new(io::ErrorKind::Other, error)),
        }
    }

    #[inline]
    pub(crate) fn flush<Inner: Write>(
        internals: &mut impl Utf8WriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.inner_mut().flush()
    }
}
