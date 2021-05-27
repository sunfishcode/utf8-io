use std::io::{self, Write};
use utf8_io::Utf8Writer;

#[test]
fn incomplete_c() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xc2").unwrap();
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn incomplete_d() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xd0").unwrap();
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn incomplete_e() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xe1").unwrap();
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn incomplete_f() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xf1").unwrap();
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn not_yet_complete_e() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xe1").unwrap();
    writer.write_all(b"\x80").unwrap();
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn not_yet_complete_f() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xf1").unwrap();
    writer.write_all(b"\x80").unwrap();
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn still_not_yet_complete_f() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xf1").unwrap();
    writer.write_all(b"\x80").unwrap();
    writer.write_all(b"\x80").unwrap();
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn complete_c() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xc2").unwrap();
    writer.write_all(b"\x80world").unwrap();
    writer.flush().unwrap();
    assert_eq!(&writer.into_inner().unwrap(), b"hello\xc2\x80world");
}

#[test]
fn complete_d() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xd0").unwrap();
    writer.write_all(b"\x80world").unwrap();
    writer.flush().unwrap();
    assert_eq!(&writer.into_inner().unwrap(), b"hello\xd0\x80world");
}

#[test]
fn complete_e() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xe1").unwrap();
    writer.write_all(b"\x80").unwrap();
    writer.write_all(b"\x80world").unwrap();
    writer.flush().unwrap();
    assert_eq!(&writer.into_inner().unwrap(), b"hello\xe1\x80\x80world");
}

#[test]
fn complete_f() {
    let mut writer = Utf8Writer::new(Vec::new());
    writer.write_all(b"hello\xf1").unwrap();
    writer.write_all(b"\x80").unwrap();
    writer.write_all(b"\x80").unwrap();
    writer.write_all(b"\x80world").unwrap();
    writer.flush().unwrap();
    assert_eq!(&writer.into_inner().unwrap(), b"hello\xf1\x80\x80\x80world");
}
