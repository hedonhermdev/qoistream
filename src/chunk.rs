use nom::{
    bits::streaming::tag as bits_tag,
    bits::streaming::take as bits_take,
    bytes::streaming::{tag, take},
    error::{make_error, ErrorKind, ParseError, VerboseError},
    number::streaming::be_u8,
    sequence::tuple,
    Err as NomErr,
};

use crate::NomRes;

const QOI_RGB_CHUNK_TAG: u8 = 0b11111110;
const QOI_RGBA_CHUNK_TAG: u8 = 0b11111111;
const QOI_OP_INDEX_CHUNK_TAG: usize = 0b00;
const QOI_OP_DIFF_CHUNK_TAG: usize = 0b01;
const QOI_OP_LUMA_CHUNK_TAG: u8 = 0b10;
const QOI_OP_RUN_CHUNK_TAG: usize = 0b11;

#[derive(Debug, PartialEq, Eq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OpIndex {
    pub index: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OpDiff {
    pub diff_r: u8,
    pub diff_g: u8,
    pub diff_b: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OpLuma {
    pub diff_g: u8,
    pub dr_dg: u8,
    pub db_dg: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OpRun {
    pub run: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum QOIChunk {
    RGB(RGB),
    RGBA(RGBA),
    OpIndex(OpIndex),
    OpDiff(OpDiff),
    OpLuma(OpLuma),
    OpRun(OpRun),
    EndMarker,
}

pub fn parse_rgb_chunk(input: &[u8]) -> NomRes<&[u8], QOIChunk> {
    let (i, tag) = be_u8(input)?;

    if tag != QOI_RGB_CHUNK_TAG {
        return Err(NomErr::Error(make_error(i, ErrorKind::Tag)));
    }

    let (i, (r, g, b)) = tuple((be_u8, be_u8, be_u8))(i)?;

    let chunk = RGB { r, g, b };

    Ok((i, QOIChunk::RGB(chunk)))
}

pub fn parse_rgba_chunk(input: &[u8]) -> NomRes<&[u8], QOIChunk> {
    let (i, tag) = be_u8(input)?;

    if tag != QOI_RGBA_CHUNK_TAG {
        return Err(NomErr::Error(make_error(i, ErrorKind::Tag)));
    }

    let (i, (r, g, b, a)) = tuple((be_u8, be_u8, be_u8, be_u8))(i)?;
    let chunk = RGBA { r, g, b, a };

    Ok((i, QOIChunk::RGBA(chunk)))
}

pub fn parse_op_index_chunk(input: &[u8]) -> NomRes<&[u8], QOIChunk> {
    let ((i, offset), (_, index)) = tuple::<_, _, VerboseError<(&[u8], usize)>, _>((
        bits_tag(QOI_OP_INDEX_CHUNK_TAG, 2usize),
        bits_take(6usize),
    ))((input, 0))
    .map_err(|_| NomErr::Error(VerboseError::from_error_kind(input, ErrorKind::Fail)))?;

    assert_eq!(offset, 0);

    let chunk = OpIndex { index };

    Ok((i, QOIChunk::OpIndex(chunk)))
}

pub fn parse_op_diff_chunk(input: &[u8]) -> NomRes<&[u8], QOIChunk> {
    let ((i, offset), (_, diff_r, diff_g, diff_b)) =
        tuple::<_, _, VerboseError<(&[u8], usize)>, _>((
            bits_tag(QOI_OP_DIFF_CHUNK_TAG, 2usize),
            bits_take(2usize),
            bits_take(2usize),
            bits_take(2usize),
        ))((input, 0))
        .map_err(|_| NomErr::Error(VerboseError::from_error_kind(input, ErrorKind::Fail)))?;

    assert_eq!(offset, 0);

    let chunk = OpDiff {
        diff_r,
        diff_g,
        diff_b,
    };

    Ok((i, QOIChunk::OpDiff(chunk)))
}

pub fn parse_op_luma_chunk(input: &[u8]) -> NomRes<&[u8], QOIChunk> {
    let (i, bytes) = take(2usize)(input)?;

    let ((_, offset), (_, diff_g, dr_dg, db_dg)) =
        tuple::<_, _, VerboseError<(&[u8], usize)>, _>((
            bits_tag(QOI_OP_LUMA_CHUNK_TAG, 2u8),
            bits_take(6usize),
            bits_take(4usize),
            bits_take(4usize),
        ))((bytes, 0))
        .map_err(|_| NomErr::Error(VerboseError::from_error_kind(input, ErrorKind::Fail)))?;

    assert_eq!(offset, 0);

    let chunk = OpLuma {
        diff_g,
        dr_dg,
        db_dg,
    };

    Ok((i, QOIChunk::OpLuma(chunk)))
}

pub fn parse_op_run_chunk(input: &[u8]) -> NomRes<&[u8], QOIChunk> {
    let ((i, offset), (_, run)) =
        tuple::<_, _, VerboseError<(&[u8], usize)>, _>((
            bits_tag(QOI_OP_RUN_CHUNK_TAG, 2u8),
            bits_take(6usize),
        ))((input, 0))
        .map_err(|_| NomErr::Error(VerboseError::from_error_kind(input, ErrorKind::Fail)))?;

    assert_eq!(offset, 0);

    let chunk = OpRun { run };

    Ok((i, QOIChunk::OpRun(chunk)))
}

pub fn parse_end_marker(input: &[u8]) -> NomRes<&[u8], QOIChunk> {
    let (i, _) = tag([0, 0, 0, 0, 0, 0, 0, 1])(input)?;

    Ok((i, QOIChunk::EndMarker))
}