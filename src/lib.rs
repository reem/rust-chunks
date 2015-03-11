#![feature(io, core)]
#![cfg_attr(test, deny(warnings))]
//#![deny(missing_docs)]

//! # chunks
//!
//! A library for dealing in chunky streams of data.
//!

extern crate syncbox;
extern crate bytes;
extern crate mio;

pub use syncbox::util::async::Future;
use bytes::traits::*;
use bytes::{SliceBuf, MutSliceBuf};
use std::io;

pub trait Chunks: Sized + Send {
    type Reader: ChunkReader<Self>;

    fn chunk(self) -> Future<Self::Reader, io::Error>;

    fn pipe<W: Consumer>(self, write: W) -> Future<(Self, W), io::Error> {
        pipe::pipe(self, write)
    }
}

pub trait ChunkReader<R: Chunks>: Sized + Send {
    fn read<M: MutBuf>(self, &mut M) -> io::Result<(usize, R)>;
    fn read_slice(self, buf: &mut [u8]) -> io::Result<(usize, R)> {
        self.read(&mut MutSliceBuf::wrap(buf))
    }
}

pub trait Consumer: Sized + Send {
    type Writer: ChunkConsumer<Self>;

    fn chunk(self) -> Future<Self::Writer, io::Error>;

    fn consume<R: Chunks>(self, read: R) -> Future<(R, Self), io::Error> {
        read.pipe(self)
    }
}

pub trait ChunkConsumer<W: Consumer>: Sized + Send {
    fn write<B: Buf>(self, &mut B) -> io::Result<(usize, W)>;

    fn write_slice(self, buf: &[u8]) -> io::Result<(usize, W)> {
        self.write(&mut SliceBuf::wrap(buf))
    }
}

mod pipe;
pub mod util;
pub mod impls;

