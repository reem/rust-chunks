use std::io::{self, Read, Write};
use std::slice::bytes::copy_memory;

use bytes::traits::*;
use syncbox::util::async::join;

use {Chunks, Consumer, ChunkReader, ChunkConsumer, Future};

#[derive(Debug, Default, Clone, Hash)]
pub struct MemReader {
    pos: usize,
    buf: Vec<u8>
}

impl io::Read for MemReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let out_amount = self.buf.len() - self.pos;
        let in_space = buf.len();

        let read_amount = if out_amount > in_space {
            copy_memory(buf, &self.buf[self.pos .. self.pos + in_space]);
            self.pos += in_space;
            in_space
        } else {
            copy_memory(buf, &self.buf[self.pos..]);
            buf.len() - self.pos
        };

        Ok(read_amount)
    }
}

#[derive(Debug, Default, Clone, Hash)]
pub struct MemChunkReader(MemReader);

impl Chunks for MemReader {
    type Reader = MemChunkReader;

    fn chunk(self) -> Future<MemChunkReader, io::Error> {
        Future::of(MemChunkReader(self))
    }
}

impl ChunkReader<MemReader> for MemChunkReader {
    fn read<M: MutBuf>(mut self, buf: &mut M) -> io::Result<(usize, MemReader)> {
        let read = try!(io::Read::read(&mut self.0, buf.mut_bytes()));
        buf.advance(read);
        Ok((read, self.0))
    }
}

impl Consumer for Vec<u8> {
    type Writer = MemChunkConsumer;

    fn chunk(self) -> Future<MemChunkConsumer, io::Error> {
        Future::of(MemChunkConsumer(self))
    }
}

#[derive(Debug, Default, Clone, Hash)]
pub struct MemChunkConsumer(Vec<u8>);

impl ChunkConsumer<Vec<u8>> for MemChunkConsumer {
    fn write<B: Buf>(mut self, buf: &mut B) -> io::Result<(usize, Vec<u8>)> {
        let wrote = try!(io::Write::write(&mut self.0, buf.bytes()));
        buf.advance(wrote);
        Ok((wrote, self.0))
    }
}

pub struct Broadcast<T, U>(T, U);
pub struct BroadcastChunk<T, U>(T, U);

impl<T, U> Consumer for Broadcast<T, U>
where T: Consumer, U: Consumer {
    type Writer = BroadcastChunk<T::Writer, U::Writer>;

    fn chunk(self) -> Future<BroadcastChunk<T::Writer, U::Writer>, io::Error> {
        join((self.0.chunk(), self.1.chunk())).map(move |(reader, writer)| {
            BroadcastChunk(reader, writer)
        })
    }
}

impl<T, U> ChunkConsumer<Broadcast<T, U>> for BroadcastChunk<T::Writer, U::Writer>
where T: Consumer, U: Consumer {
    fn write<B: Buf>(self, buf: &mut B) -> io::Result<(usize, Broadcast<T, U>)> {
        // FIXME: Behavior here is not ideal, might need to introduce buffer.
        let (wrote, next1) = try!(self.0.write_slice(buf.bytes()));
        let (_, next2) = try!(self.1.write_slice(&buf.bytes()[..wrote]));

        buf.advance(wrote);
        Ok((wrote, Broadcast(next1, next2)))
    }
}

