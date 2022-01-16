use core::panic;
use std::any::type_name;
use std::fs::File;
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

use circular::Buffer;
use nom::{
    branch::alt,
    bytes::streaming::tag,
    error::context,
    number::streaming::{be_u32, be_u8},
    sequence::tuple,
    IResult, Needed, Offset,
};
use std::io::Read;

use crate::chunk::*;
use crate::header::QOIHeader;
use crate::NomRes;

pub struct QOIParser {
    file: File,
    buffer: Buffer,
    capacity: usize,
}

impl QOIParser {
    pub fn new(file: File) -> Self {
        let capacity = 17;
        let buffer = Buffer::with_capacity(capacity);

        Self {
            file,
            buffer,
            capacity,
        }
    }

    fn parse_header(bytes: &[u8]) -> IResult<&[u8], QOIHeader> {
        let (i, (_, width, height, channels, colorspace)) = context(
            "failed to parse header",
            tuple((tag("qoif"), be_u32, be_u32, be_u8, be_u8)),
        )(bytes)?;

        let header = QOIHeader::new(width, height, channels, colorspace);

        Ok((i, header))
    }

    fn parse_chunk(bytes: &[u8]) -> NomRes<&[u8], QOIChunk> {
        context(
            "failed to parse chunk",
            alt((
                parse_end_marker,
                parse_op_index_chunk,
                parse_rgb_chunk,
                parse_rgba_chunk,
                parse_op_diff_chunk,
                parse_op_luma_chunk,
                parse_op_run_chunk,
            )),
        )(bytes)
    }

    pub fn parse(&mut self) -> std::io::Result<(QOIHeader, impl Generator + '_)> {
        let sz = self.file.read(self.buffer.space()).expect("should write");
        self.buffer.fill(sz);

        let res = Self::parse_header(self.buffer.data());

        let (header, length) = if let IResult::Ok((remaining, header)) = res {
            // `offset()` is a helper method of `nom::Offset` that can compare two slices and indicate
            // how far they are from each other. The parameter of `offset()` must be a subset of the
            // original slice
            (header, self.buffer.data().offset(remaining))
        } else {
            // TODO: do not panic on error
            panic!("couldn't parse header");
        };

        self.buffer.consume(length);

        let mut generator = move || {
            let mut consumed = length;

            loop {
                let sz = self.file.read(self.buffer.space()).expect("should write");
                self.buffer.fill(sz);

                if self.buffer.available_data() == 0 {
                    println!("no more data to read or parse, stopping the reading loop");
                    break;
                }

                let needed: Option<Needed>;

                loop {
                    let (length, chunk) = {
                        match Self::parse_chunk(self.buffer.data()) {
                            Ok((remaining, chunk)) => (self.buffer.data().offset(remaining), chunk),
                            Err(e) => match e {
                                nom::Err::Incomplete(n) => {
                                    needed = Some(n);
                                    break;
                                }
                                nom::Err::Error(e) => {
                                    panic!(
                                        "error in parsing chunk: {:?} consumed: {}",
                                        e, consumed
                                    );
                                }
                                nom::Err::Failure(e) => {
                                    panic!(
                                        "failure in parsing chunk: {:?} consumed: {}",
                                        e, consumed
                                    );
                                }
                            },
                        }
                    };

                    self.buffer.consume(length);
                    consumed += length;

                    yield chunk;
                }

                if let Some(Needed::Size(sz)) = needed {
                    let req: usize = sz.into();
                    if req > self.capacity {
                        self.capacity *= 2;
                        self.buffer.grow(self.capacity);
                    }
                }
            }
        };

        print_type_of(&generator);

        Ok((header, generator))
    }
}