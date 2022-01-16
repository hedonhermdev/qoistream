#![feature(generators, generator_trait)]

mod decoder;
mod chunk;
mod header;
mod parser;

pub use crate::header::QOIHeader;
pub use crate::chunk::QOIChunk;
pub use crate::parser::QOIParser;

use nom::{IResult, error::VerboseError};

pub(crate) type NomRes<T, U> = IResult<T, U, VerboseError<T>>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}