#![cfg_attr(test, deny(warnings))]
//#![deny(missing_docs)]

//! # chunks
//!
//! A library for dealing in chunky streams of data.
//!

extern crate syncbox;
extern crate rcmut;
extern crate bytes;
extern crate mio;

pub use syncbox::util::async::Future;
use bytes::traits::*;

pub trait Chunks: Sized + Send {
    type Error: Send;
    type Reader: ChunkReader<Self>;

    fn chunk(self) -> Future<Self::Reader, Self::Error>;

    fn pipe<W: Consumer<Error=Self::Error>>(self, write: W) -> Future<(), Self::Error> {
        pipe::pipe(self, write)
    }
}

pub trait ChunkReader<R: Chunks>: Sized + Send {
    fn read<M: MutBuf>(self, &mut M) -> Result<(usize, Option<R>), R::Error>;
}

pub trait Consumer: Sized + Send {
    type Error: Send;
    type Writer: ChunkConsumer<Self>;

    fn chunk(self) -> Future<Self::Writer, Self::Error>;

    fn consume<R: Chunks<Error=Self::Error>>(self, read: R) -> Future<(), R::Error> {
        read.pipe(self)
    }
}

pub trait ChunkConsumer<W: Consumer>: Sized + Send {
    fn write<B: Buf>(self, &mut B) -> Result<(usize, Option<W>), W::Error>;
}

mod pipe;

