use std::{
    fmt::{self, Arguments},
    io::{self, Write},
};

/// Add a convenience and optimizing method for writing from `str`.
pub trait WriteStr: Write {
    /// Like [`Write::write_all`], but takes a `&str`, allowing implementors
    /// which require valid UTF-8 to avoid re-validating the data.
    #[inline]
    fn write_str(&mut self, buf: &str) -> io::Result<()> {
        default_write_str(self, buf)
    }
}

/// Default implementation of [`WriteStr::write_str`], in terms of
/// [`Write::write_all`].
#[inline]
pub fn default_write_str<Inner: Write + ?Sized>(inner: &mut Inner, buf: &str) -> io::Result<()> {
    // Default to just writing it as bytes.
    inner.write_all(buf.as_bytes())
}

/// Default implementation of [`Write::write_fmt`], in terms of
/// [`WriteStr::write_str`].
pub fn default_write_fmt<Inner: WriteStr + ?Sized>(
    inner: &mut Inner,
    fmt: Arguments,
) -> io::Result<()> {
    struct Adaptor<'a, Inner: ?Sized + 'a> {
        inner: &'a mut Inner,
        error: Option<io::Error>,
    }

    impl<Inner: WriteStr + ?Sized> fmt::Write for Adaptor<'_, Inner> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            match self.inner.write_str(s) {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.error = Some(e);
                    Err(fmt::Error)
                }
            }
        }
    }

    let mut adapter = Adaptor { inner, error: None };
    match fmt::write(&mut adapter, fmt) {
        Ok(()) => Ok(()),
        Err(_) => Err(adapter
            .error
            .unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "formatter error"))),
    }
}
