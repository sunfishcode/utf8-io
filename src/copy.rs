use crate::{ReadStr, WriteStr};
use std::io;
#[cfg(feature = "layered-io")]
use {crate::ReadStrLayered, layered_io::Bufferable, std::cmp::max};

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

/// Like `std::io::copy`, but for streams that can operate directly on strings,
/// so we can avoid re-validating them as UTF-8.
pub fn copy_str<R: ReadStr + ?Sized, W: WriteStr + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> io::Result<u64> {
    // TODO: Avoid unnecessary zero-initialization.
    let mut buf = "\0".repeat(DEFAULT_BUF_SIZE);

    let mut written = 0;
    loop {
        let len = match reader.read_str(&mut buf) {
            Ok(0) => break,
            Ok(nread) => nread,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        };
        writer.write_str(&buf[..len])?;
        written += len as u64;
    }
    Ok(written)
}

/// Like `std::io::copy`, but for streams that can operate directly on strings,
/// so we can avoid re-validating them as UTF-8.
///
/// Also, like `copy_str`, but uses `read_str_with_status` to avoid performing
/// an extra `read` at the end.
#[cfg(feature = "layered-io")]
pub fn copy_str_using_status<
    R: ReadStrLayered + Bufferable + ?Sized,
    W: WriteStr + Bufferable + ?Sized,
>(
    reader: &mut R,
    writer: &mut W,
) -> io::Result<u64> {
    // TODO: Avoid unnecessary zero-initialization.
    let mut buf = "\0".repeat(max(
        reader.suggested_buffer_size(),
        writer.suggested_buffer_size(),
    ));

    let mut written = 0;
    loop {
        let (len, status) = reader.read_str_with_status(&mut buf)?;
        writer.write_str(&buf[..len])?;
        written += len as u64;
        if status.is_end() {
            return Ok(written);
        }
        if status.is_push() {
            writer.flush()?;
        }
    }
}

#[test]
fn test_copy_str() {
    use crate::{Utf8Reader, Utf8Writer};
    use std::{io::Cursor, str};

    let text = "hello world ☃";
    let mut input = Utf8Reader::new(Cursor::new(text.to_string()));
    let mut output = Utf8Writer::new(Vec::new());

    copy_str(&mut input, &mut output).unwrap();

    let vec = output.into_inner().unwrap();
    assert_eq!(str::from_utf8(&vec).unwrap(), text);
}

#[cfg(feature = "layered-io")]
#[test]
fn test_copy_str_using_status() {
    use crate::{Utf8Reader, Utf8Writer};
    use layered_io::{LayeredReader, LayeredWriter};
    use std::{io::Cursor, str};

    let text = "hello world ☃";
    let mut input = Utf8Reader::new(LayeredReader::new(Cursor::new(text.to_string())));
    let mut output = Utf8Writer::new(LayeredWriter::new(Vec::new()));

    copy_str_using_status(&mut input, &mut output).unwrap();

    let ext = output.close_into_inner().unwrap();
    let vec = ext.abandon_into_inner().unwrap();
    assert_eq!(str::from_utf8(&vec).unwrap(), text);
}
