use std::fs::File;

use qoistream::QOIParser;

fn main() -> std::io::Result<()> {
    let file = File::open("./images/baboon.qoi")?;
    let mut parser = QOIParser::new(file);

    parser.parse()?;

    Ok(())
}