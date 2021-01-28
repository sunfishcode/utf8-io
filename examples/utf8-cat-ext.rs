#[cfg(feature = "layered-io")]
use {
    layered_io::WriteLayered,
    layered_io::{LayeredReader, LayeredWriter},
    utf8_io::{copy_str, Utf8Reader, Utf8Writer},
};

#[cfg(feature = "layered-io")]
fn main() -> anyhow::Result<()> {
    let mut reader = Utf8Reader::new(LayeredReader::new(std::io::stdin()));
    let mut writer = Utf8Writer::new(LayeredWriter::new(std::io::stdout()));
    copy_str(&mut reader, &mut writer)?;
    writer.close()?;
    Ok(())
}

#[cfg(not(feature = "layered-io"))]
fn main() -> anyhow::Result<()> {
    panic!("The utf8-cat-ext example requires the layered-io feature.")
}
