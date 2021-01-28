//! Traits and types for UTF-8 I/O

#![deny(missing_docs)]

mod copy;
mod read_str;
mod utf8_duplexer;
mod utf8_input;
mod utf8_output;
mod utf8_reader;
mod utf8_writer;
mod write_str;

pub use copy::copy_str;
#[cfg(feature = "layered-io")]
pub use copy::copy_str_using_status;
#[cfg(feature = "layered-io")]
pub use read_str::ReadStrLayered;
pub use read_str::{default_read_exact_str, ReadStr};
pub use utf8_duplexer::Utf8Duplexer;
pub use utf8_reader::Utf8Reader;
pub use utf8_writer::Utf8Writer;
pub use write_str::{default_write_fmt, default_write_str, WriteStr};
