use std::fs::File;

use qoistream::QOIParser;
use qoistream::QOIDecoder;

fn main() -> std::io::Result<()> {
    let file = File::open("./images/baboon.qoi")?;
    let parser = QOIParser::new(file);

    let (header, generator) = parser.parse()?;

    let decoder = QOIDecoder::new(header, generator);

    let _buffer = decoder.decode();

    Ok(())
}