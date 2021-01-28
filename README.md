<div align="center">
  <h1><code>utf8-io</code></h1>

  <p>
    <strong>Traits and types for UTF-8 I/O</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/utf8-io/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/utf8-io/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://crates.io/crates/utf8-io"><img src="https://img.shields.io/crates/v/utf8-io.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/utf8-io"><img src="https://docs.rs/utf8-io/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

`utf8-io` defines several utilities for performing UTF-8 I/O.

 - [`ReadStr`] and [`WriteStr`] are traits which extend [`Read`] and [`Write`]
   providing `read_str` and `write_str` functions for reading and writing UTF-8
   data.

 - [`Utf8Reader`] and [`Utf8Writer`] implement `ReadStr` and `WriteStr` and
   wrap arbitrary `Read` and `Write` implementations. `Utf8Reader` translates
   invalid UTF-8 encodings into replacements (U+FFFD), while `Utf8Writer`
   reports errors on invalid UTF-8 encodings. Both ensure that scalar values
   are never split at the end of a buffer.

 - [`Utf8Duplexer`] represents an interactive stream and implements both
   `ReadStr` and `WriteStr`.

[`ReadStr`]: https://docs.rs/utf8-io/latest/utf8_io/trait.ReadStr.html
[`WriteStr`]: https://docs.rs/utf8-io/latest/utf8_io/trait.WriteStr.html
[`Utf8Reader`]: https://docs.rs/utf8-io/latest/utf8_io/struct.Utf8Reader.html
[`Utf8Writer`]: https://docs.rs/utf8-io/latest/utf8_io/struct.Utf8Writer.html
[`Utf8Duplexer`]: https://docs.rs/utf8-io/latest/utf8_io/struct.Utf8Duplexer.html
