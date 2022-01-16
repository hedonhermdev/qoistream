use std::{ops::{Generator, GeneratorState}, pin::Pin};
use crate::{QOIChunk, QOIHeader};

use crossbeam::{thread::scope, channel::{Sender, Receiver, bounded}};

trait ChunkGeneratorTrait: Generator<Return = bool, Yield = QOIChunk> + Send {}
type ChunkGenerator<'a> = &'a mut (dyn ChunkGeneratorTrait + 'a);

struct ChunkDecoder<'a> {
    generator: ChunkGenerator<'a>,
    buffer: Vec<u32>,
}

impl<'a> ChunkDecoder<'a> {
    fn new(header: QOIHeader, generator: ChunkGenerator<'a> ) -> Self {
        let capacity = (header.height() * header.width()) as usize;

        let buffer = vec![0; capacity];

        Self {
            generator,
            buffer,
        }
    }

    fn decode(self) {
        crossbeam::thread::scope(|s| {
            let (sender, receiver) = bounded::<Option<QOIChunk>>(4);

            let decode_thread = s.spawn(move |_| {
                while let Some(chunk) = receiver.recv().expect("failed to receive from channel") {
                    println!("chunk recvd: {:?}", chunk);   
                }
            });

            let generator_thread = s.spawn(move |_| {
                // loop {
                //     match generator.resume(()) {
                //         GeneratorState::Yielded(chunk) => {
                //             println!("{:?}", chunk);
                //         }
                //         GeneratorState::Complete(_) => {
                //             break;
                //         }
                //     }
                // }
            });
        }).expect("failed to spawn scoped thread");
    }
}