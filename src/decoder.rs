// Copyright (c) 2015, Sam Payson
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
// associated documentation files (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify, merge, publish, distribute,
// sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
// NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::error::FromError;
use std::io;
use std::mem;
use std::result;
use std::io::ReadExt;

use iter::{RexIterExt, ResultIterExt};

/// `Error` is used to report errors that occur within the decoder.
pub enum Error {
    /// `EOF` indicates that record source ended before it could be fully decoded.
    EOF,

    /// `IoError` is used to pass through std::io errors.
    IoError(io::Error),
}

impl FromError<io::Error> for Error {
    fn from_error(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

/// `read_uvarint` reads a varint encoded `u64` from `r`.
fn read_uvarint<R>(r: &mut R) -> Result<u64, Error>
    where R: io::Read {

    let itr = r.bytes()
        .take_while_incl(|x| match x { &Ok(x) => x >= 0x80, _ => true });

    // The Ok(try!(..)) just lets us convert from io::Error to Error
    result::fold(itr, 0, |sum, x| (sum << 7) + (x & 0x7F) as u64)
        .map_err(FromError::from_error)
}

/// `read_varint` reads a zig-zag varint encoded `i64` from `r`.
fn read_varint<R>(r: &mut R) -> Result<i64, Error>
    where R: io::Read {

    let ux = try!(read_uvarint(r));

    Ok( if ux & 1 != 0 {
        let x = (!ux >> 1) as i64;
        -x
    } else {
        let x = (ux >> 1) as i64;
        x
    })
}

/// `read_u8` reads a single byte from `r`.
fn read_u8<R>(r: &mut R) -> Result<u8, Error>
    where R: io::Read {

    r.bytes()
        .take_or_err(1, Error::EOF)
        .next()
        // take_or_err guarantees that we'll get an Error at worst.
        .unwrap()
}

/// `read_le_u16` reads 2 bytes as a little endian `u16` from 'r'
fn read_le_u16<R>(r: &mut R) -> Result<u16, Error>
    where R: io::Read {

    let itr = r.bytes().take_or_err(2, Error::EOF);

    result::fold(itr, 0, |so_far, next| (so_far << 8) + next as u16)
}

/// `read_le_u32` reads 4 bytes as a little endian `u32` from 'r'
fn read_le_u32<R>(r: &mut R) -> Result<u32, Error>
    where R: io::Read {

    let itr = r.bytes().take_or_err(4, Error::EOF);

    result::fold(itr, 0, |so_far, next| (so_far << 8) + next as u32)
}

/// `read_le_u64` reads 8 bytes as a little endian `u64` from 'r'
fn read_le_u64<R>(r: &mut R) -> Result<u64, Error>
    where R: io::Read {

    let itr = r.bytes().take_or_err(8, Error::EOF);

    result::fold(itr, 0, |so_far, next| (so_far << 8) + next as u64)
}

/// `read_i8` reads a single byte from `r`, as a 2's complement `i8`
fn read_i8<R>(r: &mut R) -> Result<i8, Error>
    where R: io::Read {

    Ok(try!(read_u8(r)) as i8)
}

/// `read_le_i16` reads 2 bytes as a little endian 2's complement `i16` from `r`.
fn read_le_i16<R>(r: &mut R) -> Result<i16, Error>
    where R: io::Read {

    Ok(try!(read_le_u16(r)) as i16)
}

/// `read_le_i32` reads 4 bytes as a little endian 2's complement `i32` from `r`.
fn read_le_i32<R>(r: &mut R) -> Result<i32, Error>
    where R: io::Read {

    Ok(try!(read_le_u32(r)) as i32)
}

/// `read_le_i64` reads 8 bytes as a little endian 2's complement `i64` from `r`.
fn read_le_i64<R>(r: &mut R) -> Result<i64, Error>
    where R: io::Read {

    Ok(try!(read_le_u64(r)) as i64)
}


/// `read_le_f32` reads 8 bytes as a little endian ieee-754 binary32 encoded `f32` from `r`.
fn read_le_f32<R>(r: &mut R) -> Result<f32, Error>
    where R: io::Read {

    let u = try!(read_le_u32(r));

    Ok(unsafe { mem::transmute(u) })
}

/// `read_le_f64` reads 8 bytes as a little endian ieee-754 binary64 encoded `f64` from `r`.
fn read_le_f64<R>(r: &mut R) -> Result<f64, Error>
    where R: io::Read {

    let u = try!(read_le_u64(r));

    Ok(unsafe { mem::transmute(u) })
}
