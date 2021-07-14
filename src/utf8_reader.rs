use crate::{utf8_input::Utf8Input, ReadStr};
use std::{
    fmt,
    io::{self, Read},
    str,
};
#[cfg(feature = "terminal-io")]
use terminal_io::{ReadTerminal, Terminal};
#[cfg(windows)]
use unsafe_io::os::windows::{
    AsHandleOrSocket, AsRawHandleOrSocket, BorrowedHandleOrSocket, RawHandleOrSocket,
};
#[cfg(feature = "layered-io")]
use {
    crate::ReadStrLayered,
    layered_io::{Bufferable, ReadLayered, Status},
};
#[cfg(not(windows))]
use {
    io_lifetimes::{AsFd, BorrowedFd},
    unsafe_io::os::posish::{AsRawFd, RawFd},
};

/// A [`Read`] implementation which translates from an input `Read` producing
/// an arbitrary byte sequence into a valid UTF-8 sequence with invalid
/// sequences replaced by [U+FFFD (REPLACEMENT CHARACTER)] in the manner of
/// [`String::from_utf8_lossy`], where scalar value encodings never straddle
/// `read` calls (callers can do [`str::from_utf8`] and it will always
/// succeed).
///
/// [U+FFFD (REPLACEMENT CHARACTER)]: https://util.unicode.org/UnicodeJsps/character.jsp?a=FFFD
pub struct Utf8Reader<Inner: Read> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// UTF-8 translation state.
    pub(crate) input: Utf8Input,
}

impl<Inner: Read> Utf8Reader<Inner> {
    /// Construct a new instance of `Utf8Reader` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            input: Utf8Input::new(),
        }
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: Read + ReadTerminal> Terminal for Utf8Reader<Inner> {}

#[cfg(feature = "terminal-io")]
impl<Inner: Read + ReadTerminal> ReadTerminal for Utf8Reader<Inner> {
    #[inline]
    fn is_line_by_line(&self) -> bool {
        self.inner.is_line_by_line()
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        self.inner.is_input_terminal()
    }
}

#[cfg(feature = "layered-io")]
impl<Inner: ReadLayered> ReadLayered for Utf8Reader<Inner> {
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
impl<Inner: ReadLayered> Bufferable for Utf8Reader<Inner> {
    #[inline]
    fn abandon(&mut self) {
        Utf8Input::abandon(self)
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        Utf8Input::suggested_buffer_size(self)
    }
}

impl<Inner: Read> ReadStr for Utf8Reader<Inner> {
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
impl<Inner: ReadLayered> ReadStrLayered for Utf8Reader<Inner> {
    #[inline]
    fn read_str_with_status(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        Utf8Input::read_str_with_status(self, buf)
    }

    #[inline]
    fn read_exact_str_using_status(&mut self, buf: &mut str) -> io::Result<Status> {
        Utf8Input::read_exact_str_using_status(self, buf)
    }
}

impl<Inner: Read> Read for Utf8Reader<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Utf8Input::read(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        Utf8Input::read_to_string(self, buf)
    }
}

#[cfg(not(windows))]
impl<Inner: Read + AsRawFd> AsRawFd for Utf8Reader<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(not(windows))]
impl<Inner: Read + AsFd> AsFd for Utf8Reader<Inner> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

#[cfg(windows)]
impl<Inner: Read + AsRawHandleOrSocket> AsRawHandleOrSocket for Utf8Reader<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl<Inner: Read + AsHandleOrSocket> AsHandleOrSocket for Utf8Reader<Inner> {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.inner.as_handle_or_socket()
    }
}

impl<Inner: Read + fmt::Debug> fmt::Debug for Utf8Reader<Inner> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("Utf8Reader");
        b.field("inner", &self.inner);
        b.finish()
    }
}

#[cfg(test)]
fn translate_via_reader(bytes: &[u8]) -> String {
    let mut reader = Utf8Reader::new(bytes);
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[cfg(test)]
fn translate_via_layered_reader(bytes: &[u8]) -> String {
    let mut reader = Utf8Reader::new(layered_io::LayeredReader::new(bytes));
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[cfg(test)]
fn translate_via_slice_reader(bytes: &[u8]) -> String {
    let mut reader = Utf8Reader::new(layered_io::SliceReader::new(bytes));
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[cfg(test)]
#[cfg(feature = "layered-io")]
fn translate_with_small_buffer(bytes: &[u8]) -> String {
    let mut reader = Utf8Reader::new(layered_io::SliceReader::new(bytes));
    let mut v = Vec::new();
    let mut buf = [0; 4];
    loop {
        let (size, status) = reader.read_with_status(&mut buf).unwrap();
        v.extend_from_slice(&buf[..size]);
        if status.is_end() {
            break;
        }
    }
    String::from_utf8(v).unwrap()
}

#[cfg(test)]
#[cfg(not(feature = "layered-io"))]
fn translate_with_small_buffer(bytes: &[u8]) -> String {
    let mut reader = Utf8Reader::new(bytes);
    let mut v = Vec::new();
    let mut buf = [0; 4];
    loop {
        let size = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(size) => size,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => 0,
            Err(err) => Err(err).unwrap(),
        };
        v.extend_from_slice(&buf[..size]);
    }
    String::from_utf8(v).unwrap()
}

#[cfg(test)]
fn test(bytes: &[u8], s: &str) {
    assert_eq!(translate_via_reader(bytes), s);
    assert_eq!(translate_via_layered_reader(bytes), s);
    assert_eq!(translate_via_slice_reader(bytes), s);
    assert_eq!(translate_with_small_buffer(bytes), s);

    for i in 1..4 {
        let mut v = vec![0_u8; i + bytes.len()];
        v[i..i + bytes.len()].copy_from_slice(bytes);
        assert_eq!(
            str::from_utf8(&translate_via_reader(&v).as_bytes()[i..]).unwrap(),
            s
        );
        assert_eq!(
            str::from_utf8(&translate_via_layered_reader(&v).as_bytes()[i..]).unwrap(),
            s
        );
        assert_eq!(
            str::from_utf8(&translate_via_slice_reader(&v).as_bytes()[i..]).unwrap(),
            s
        );
        assert_eq!(
            str::from_utf8(&translate_with_small_buffer(&v).as_bytes()[i..]).unwrap(),
            s
        );
    }
}

#[test]
fn test_empty_string() {
    test(b"", "");
}

#[test]
fn test_hello_world() {
    test(b"hello world", "hello world");
}

#[test]
fn test_embedded_invalid_byte() {
    test(b"hello\xffworld", "hello�world");
}

#[test]
fn test_invalid_bytes() {
    test(b"\xff\xff\xff", "���");
}

#[test]
fn test_some_ascii_printable() {
    test(
        b"`1234567890-=qwertyuiop[]\\asdfghjkl;\"zxcvbnm,./",
        "`1234567890-=qwertyuiop[]\\asdfghjkl;\"zxcvbnm,./",
    );
}

// Tests derived from the tests in https://hsivonen.fi/broken-utf-8/

// Non-shortest forms for lowest single-byte (U+0000)
#[test]
fn test_two_byte_sequence_lowest_single_byte() {
    test(b"\xC0\x80", "��");
}
#[test]
fn test_three_byte_sequence_lowest_single_byte() {
    test(b"\xE0\x80\x80", "���");
}
#[test]
fn test_four_byte_sequence_lowest_single_byte() {
    test(b"\xF0\x80\x80\x80", "����");
}
#[test]
fn test_five_byte_sequence_lowest_single_byte() {
    test(b"\xF8\x80\x80\x80\x80", "�����");
}
#[test]
fn test_six_byte_sequence_lowest_single_byte() {
    test(b"\xFC\x80\x80\x80\x80\x80", "������");
}

// Non-shortest forms for highest single-byte (U+007F)
#[test]
fn test_two_byte_sequence_highest_single_byte() {
    test(b"\xC1\xBF", "��");
}
#[test]
fn test_three_byte_sequence_highest_single_byte() {
    test(b"\xE0\x81\xBF", "���");
}
#[test]
fn test_four_byte_sequence_highest_single_byte() {
    test(b"\xF0\x80\x81\xBF", "����");
}
#[test]
fn test_five_byte_sequence_highest_single_byte() {
    test(b"\xF8\x80\x80\x81\xBF", "�����");
}
#[test]
fn test_six_byte_sequence_highest_single_byte() {
    test(b"\xFC\x80\x80\x80\x81\xBF", "������");
}

// Non-shortest forms for lowest two-byte (U+0080)
#[test]
fn test_three_byte_sequence_lowest_two_byte() {
    test(b"\xE0\x82\x80", "���");
}
#[test]
fn test_four_byte_sequence_lowest_two_byte() {
    test(b"\xF0\x80\x82\x80", "����");
}
#[test]
fn test_five_byte_sequence_lowest_two_byte() {
    test(b"\xF8\x80\x80\x82\x80", "�����");
}
#[test]
fn test_six_byte_sequence_lowest_two_byte() {
    test(b"\xFC\x80\x80\x80\x82\x80", "������");
}

// Non-shortest forms for highest two-byte (U+07FF)
#[test]
fn test_three_byte_sequence_highest_two_byte() {
    test(b"\xE0\x9F\xBF", "���");
}
#[test]
fn test_four_byte_sequence_highest_two_byte() {
    test(b"\xF0\x80\x9F\xBF", "����");
}
#[test]
fn test_five_byte_sequence_highest_two_byte() {
    test(b"\xF8\x80\x80\x9F\xBF", "�����");
}
#[test]
fn test_six_byte_sequence_highest_two_byte() {
    test(b"\xFC\x80\x80\x80\x9F\xBF", "������");
}

// Non-shortest forms for lowest three-byte (U+0800)
#[test]
fn test_four_byte_sequence_lowest_three_byte() {
    test(b"\xF0\x80\xA0\x80", "����");
}
#[test]
fn test_five_byte_sequence_lowest_three_byte() {
    test(b"\xF8\x80\x80\xA0\x80", "�����");
}
#[test]
fn test_six_byte_sequence_lowest_three_byte() {
    test(b"\xFC\x80\x80\x80\xA0\x80", "������");
}

// Non-shortest forms for highest three-byte (U+FFFF)
#[test]
fn test_four_byte_sequence_highest_three_byte() {
    test(b"\xF0\x8F\xBF\xBF", "����");
}
#[test]
fn test_five_byte_sequence_highest_three_byte() {
    test(b"\xF8\x80\x8F\xBF\xBF", "�����");
}
#[test]
fn test_six_byte_sequence_highest_three_byte() {
    test(b"\xFC\x80\x80\x8F\xBF\xBF", "������");
}

// Non-shortest forms for lowest four-byte (U+10000)
#[test]
fn test_five_byte_sequence_lowest_four_byte() {
    test(b"\xF8\x80\x90\x80\x80", "�����");
}
#[test]
fn test_six_byte_sequence_lowest_four_byte() {
    test(b"\xFC\x80\x80\x90\x80\x80", "������");
}

// Non-shortest forms for last Unicode (U+10FFFF)
#[test]
fn test_five_byte_sequence() {
    test(b"\xF8\x84\x8F\xBF\xBF", "�����");
}
#[test]
fn test_six_byte_sequence() {
    test(b"\xFC\x80\x84\x8F\xBF\xBF", "������");
}

// Out of range
#[test]
fn test_one_past_unicode() {
    test(b"\xF4\x90\x80\x80", "����");
}
#[test]
fn test_longest_five_byte_sequence() {
    test(b"\xFB\xBF\xBF\xBF\xBF", "�����");
}
#[test]
fn test_longest_six_byte_sequence() {
    test(b"\xFD\xBF\xBF\xBF\xBF\xBF", "������");
}
#[test]
fn test_first_surrogate() {
    test(b"\xED\xA0\x80", "���");
}
#[test]
fn test_last_surrogate() {
    test(b"\xED\xBF\xBF", "���");
}
#[test]
fn test_cesu_8_surrogate_pair() {
    test(b"\xED\xA0\xBD\xED\xB2\xA9", "������");
}

// Out of range and non-shortest
#[test]
fn test_one_past_unicode_as_five_byte_sequence() {
    test(b"\xF8\x84\x90\x80\x80", "�����");
}
#[test]
fn test_one_past_unicode_as_six_byte_sequence() {
    test(b"\xFC\x80\x84\x90\x80\x80", "������");
}
#[test]
fn test_first_surrogate_as_four_byte_sequence() {
    test(b"\xF0\x8D\xA0\x80", "����");
}
#[test]
fn test_last_surrogate_as_four_byte_sequence() {
    test(b"\xF0\x8D\xBF\xBF", "����");
}
#[test]
fn test_cesu_8_surrogate_pair_as_two_four_byte_overlongs() {
    test(b"\xF0\x8D\xA0\xBD\xF0\x8D\xB2\xA9", "��������");
}

// Lone trails
#[test]
fn test_one() {
    test(b"\x80", "�");
}
#[test]
fn test_two() {
    test(b"\x80\x80", "��");
}
#[test]
fn test_three() {
    test(b"\x80\x80\x80", "���");
}
#[test]
fn test_four() {
    test(b"\x80\x80\x80\x80", "����");
}
#[test]
fn test_five() {
    test(b"\x80\x80\x80\x80\x80", "�����");
}
#[test]
fn test_six() {
    test(b"\x80\x80\x80\x80\x80\x80", "������");
}
#[test]
fn test_seven() {
    test(b"\x80\x80\x80\x80\x80\x80\x80", "�������");
}
#[test]
fn test_after_valid_two_byte() {
    test(b"\xC2\xB6\x80", "¶�");
}
#[test]
fn test_after_valid_three_byte() {
    test(b"\xE2\x98\x83\x80", "☃�");
}
#[test]
fn test_after_valid_four_byte() {
    test(b"\xF0\x9F\x92\xA9\x80", "💩�");
}
#[test]
fn test_after_five_byte() {
    test(b"\xFB\xBF\xBF\xBF\xBF\x80", "������");
}
#[test]
fn test_after_six_byte() {
    test(b"\xFD\xBF\xBF\xBF\xBF\xBF\x80", "�������");
}

// Truncated_sequences
#[test]
fn test_two_byte_lead() {
    test(b"\xC2", "�");
}
#[test]
fn test_three_byte_lead() {
    test(b"\xE2", "�");
}
#[test]
fn test_three_byte_lead_and_one_trail() {
    test(b"\xE2\x98", "�");
}
#[test]
fn test_four_byte_lead() {
    test(b"\xF0", "�");
}
#[test]
fn test_four_byte_lead_and_one_trail() {
    test(b"\xF0\x9F", "�");
}
#[test]
fn test_four_byte_lead_and_two_trails() {
    test(b"\xF0\x9F\x92", "�");
}

// Leftovers
#[test]
fn test_fe() {
    test(b"\xFE", "�");
}

#[test]
fn test_fe_and_trail() {
    test(b"\xFE\x80", "��");
}

#[test]
fn test_ff() {
    test(b"\xFF", "�");
}
#[test]
fn test_ff_and_trail() {
    test(b"\xFF\x80", "��");
}
