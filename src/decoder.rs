use crate::{QOIChunk, QOIHeader};

use crossbeam::{channel::bounded, thread::scope};
use genawaiter::sync::GenBoxed;

const INDEX_ARRAY_LENGTH: usize = 64;

pub struct QOIDecoder {
    generator: GenBoxed<QOIChunk>,
    buffer: Vec<RawPixel>,
    state: DecoderState,
}

impl QOIDecoder {
    pub fn new(header: QOIHeader, generator: GenBoxed<QOIChunk>) -> Self {
        let capacity = (header.height() * header.width()) as usize;
        let buffer = vec![RawPixel([0, 0, 0, 255]); capacity];
        let state = DecoderState::default();

        Self {
            generator,
            buffer,
            state,
        }
    }

    pub fn decode(mut self) -> Vec<RawPixel> {
        scope(|s| {
            let (sender, receiver) = bounded::<Option<QOIChunk>>(4);

            let decode_thread = s.spawn(move |_| {
                while let Some(chunk) = receiver.recv().expect("failed to receive from channel") {
                    Self::decode_chunk(&mut self.buffer, &mut self.state, chunk);
                }

                self.buffer
            });

            s.spawn(move |_| loop {
                match self.generator.resume() {
                    genawaiter::GeneratorState::Yielded(chunk) => {
                        sender.send(Some(chunk)).expect("failed to send to channel");
                    }
                    genawaiter::GeneratorState::Complete(_) => {
                        sender.send(None).expect("failed to send to channel");
                        break;
                    }
                }
            });

            decode_thread.join().expect("failed to join thread")
        })
        .expect("failed to create scope")
    }

    fn decode_chunk(buffer: &mut [RawPixel], state: &mut DecoderState, chunk: QOIChunk) {
        let pixel = match chunk {
            QOIChunk::RGB(chunk) => {
                let pixel = Pixel::new(chunk.r, chunk.g, chunk.b, state.prev_pixel.a);
                buffer[state.index] = pixel.into();

                pixel
            }
            QOIChunk::RGBA(chunk) => {
                let pixel = Pixel::new(chunk.r, chunk.g, chunk.b, chunk.a);
                buffer[state.index] = pixel.into();

                pixel
            }
            QOIChunk::OpIndex(chunk) => {
                let pixel = state.index_arr[chunk.index as usize]
                    .expect("did not find hash at expected position");
                buffer[state.index] = pixel.into();

                pixel
            }
            QOIChunk::OpDiff(chunk) => {
                const BIAS: u8 = 2;
                let (r, g, b) = (
                    state
                        .prev_pixel
                        .r
                        .wrapping_add(chunk.diff_r)
                        .wrapping_sub(BIAS),
                    state
                        .prev_pixel
                        .g
                        .wrapping_add(chunk.diff_g)
                        .wrapping_sub(BIAS),
                    state
                        .prev_pixel
                        .b
                        .wrapping_add(chunk.diff_b)
                        .wrapping_sub(BIAS),
                );

                let pixel = Pixel::new(r, g, b, state.prev_pixel.a);
                buffer[state.index] = pixel.into();

                pixel
            }
            QOIChunk::OpLuma(chunk) => {
                const BIAS: u8 = 8;
                const GREEN_BIAS: u8 = 32;

                let vg = chunk.diff_g;

                let r = state
                    .prev_pixel
                    .r
                    .wrapping_add(vg)
                    .wrapping_add(chunk.dr_dg)
                    .wrapping_sub(BIAS);

                let g = state.prev_pixel.g.wrapping_add(vg).wrapping_sub(GREEN_BIAS);

                let b = state
                    .prev_pixel
                    .b
                    .wrapping_add(vg)
                    .wrapping_add(chunk.db_dg)
                    .wrapping_sub(BIAS);

                let pixel = Pixel::new(r, g, b, state.prev_pixel.a);
                buffer[state.index] = pixel.into();

                pixel
            }
            QOIChunk::OpRun(chunk) => {
                const BIAS: usize = 1;
                let length: usize = (chunk.run as usize).wrapping_add(BIAS);

                let pixel = state.prev_pixel;
                for _ in 0..(length) {
                    buffer[state.index] = pixel.into();
                    state.index += 1;
                }
                state.index -= 1;

                pixel
            }
            QOIChunk::EndMarker => state.prev_pixel,
        };

        state.prev_pixel = pixel;
        state.index_arr[pixel.index()] = Some(pixel);
        state.index += 1;
    }
}

struct DecoderState {
    index: usize,
    prev_pixel: Pixel,
    index_arr: [Option<Pixel>; INDEX_ARRAY_LENGTH],
}

impl Default for DecoderState {
    fn default() -> Self {
        Self {
            index: 0,
            prev_pixel: Pixel::new(0, 0, 0, 255),
            index_arr: [None; 64],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RawPixel([u8; 4]);

impl From<[u8; 4]> for RawPixel {
    fn from(bytes: [u8; 4]) -> Self {
        RawPixel(bytes)
    }
}

#[derive(Clone, Copy)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Pixel { r, g, b, a }
    }

    #[inline]
    fn index(&self) -> usize {
        let (r, g, b, a) = (
            self.r as usize,
            self.g as usize,
            self.b as usize,
            self.a as usize,
        );

        (r * 3 + g * 5 + b * 7 + a * 11) % 64
    }
}

impl Into<RawPixel> for Pixel {
    fn into(self) -> RawPixel {
        RawPixel([self.r, self.g, self.b, self.a])
    }
}

impl Into<[u8; 4]> for &RawPixel {
    fn into(self) -> [u8; 4] {
        self.0
    }
}