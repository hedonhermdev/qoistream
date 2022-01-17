mod decoder;
mod chunk;
mod header;
mod parser;

pub use crate::header::QOIHeader;
pub use crate::chunk::QOIChunk;
pub use crate::parser::QOIParser;
pub use crate::decoder::QOIDecoder;

use decoder::RawPixel;
use nom::{IResult, error::VerboseError};

pub(crate) type NomRes<T, U> = IResult<T, U, VerboseError<T>>;

pub fn read_qoi(file: std::fs::File) -> Result<(QOIHeader, Vec<RawPixel>), &'static str> {
    let parser = QOIParser::new(file);

    let (header, generator) = parser.parse().expect("failed to parse file");

    let decoder = QOIDecoder::new(header, generator);
    let buffer = decoder.decode();

    Ok((header, buffer))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}