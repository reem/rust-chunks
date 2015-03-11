use bytes::RingBuf;
use bytes::traits::*;

use std::io;

use syncbox::util::async::{Async, join};
use {Chunks, Consumer, ChunkReader, ChunkConsumer, Future};

pub fn pipe<R: Chunks, W: Consumer>(read: R, write: W) -> Future<(R, W), io::Error> {
    let buffer = RingBuf::new(8 * 1024);

    complete(read, write, buffer)
}

fn complete<
    R: Chunks, W: Consumer
>(read: R, write: W, mut buffer: RingBuf) -> Future<(R, W), io::Error> {
    join((read.chunk(), write.chunk())).and_then(move |(reader, writer)| {
        match reader.read(&mut buffer) {
            Ok((0, next_read)) => match writer.write(&mut buffer) {
                Ok((_, next_write)) => Future::of((next_read, next_write)),
                Err(e) => Future::error(e)
            },
            Ok((_, next_read)) => {
                match writer.write(&mut buffer) {
                    Ok((0, next_write)) => Future::of((next_read, next_write)),
                    Ok((_, next_write)) => complete(next_read, next_write, buffer),
                    Err(e) => Future::error(e)
                }
            },
            Err(e) => Future::error(e)
        }
    })
}

