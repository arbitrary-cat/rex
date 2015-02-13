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

use std::io;
use std::mem;

use encoding::Type;

/// A `Primitive` represents the primitive data types which make up all records. This is the format
/// used to communicate data between `Encodable`/`Decodable` types and an `Encoder`/`Decoder`.
#[allow(missing_docs)]
pub enum Primitive {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),

    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),

    Fixed32(u32),
    Fixed64(u64),

    Float32(f32),
    Float64(f64),

    Bool(bool),

    Bytes(Vec<u8>),
    String(String),

    Enum(i64),
}

impl Primitive {
    /// The `has_type` simply that the type of `self` is `t`.
    pub fn has_type(&self, t: Type) -> bool {
        use encoding::Type::*;

        match (self, t) {
            (&Primitive::UInt8(..),  UInt8)  => true,
            (&Primitive::UInt16(..), UInt8)  => true,
            (&Primitive::UInt32(..), UInt32) => true,
            (&Primitive::UInt64(..), UInt64) => true,

            (&Primitive::Int8(..),  Int8)  => true,
            (&Primitive::Int16(..), Int8)  => true,
            (&Primitive::Int32(..), Int32) => true,
            (&Primitive::Int64(..), Int64) => true,

            (&Primitive::Fixed32(..), Fixed32) => true,
            (&Primitive::Fixed64(..), Fixed64) => true,

            (&Primitive::Float32(..), Float32) => true,
            (&Primitive::Float64(..), Float64) => true,

            (&Primitive::Bool(..), Bool) => true,

            (&Primitive::Bytes(..), Bytes)   => true,
            (&Primitive::String(..), String) => true,

            (&Primitive::Enum(..), Enum) => true,

            _ => false,
        }
    }
}

/// `uvarint_size` returns the number of bytes required to encode `x` as a varint.
pub fn uvarint_size(x: u64) -> usize {
    if x < 0x80 {
        1
    } else if x < 0x80 << 7 {
        2
    } else if x < 0x80 << 14 {
        3
    } else if x < 0x80 << 21 {
        4
    } else if x < 0x80 << 28 {
        5
    } else if x < 0x80 << 35 {
        6
    } else if x < 0x80 << 42 {
        7
    } else if x < 0x80 << 49 {
        8
    } else if x < 0x80 << 56 {
        9
    } else {
        10
    }
}

/// `varint_size` returns the number of bytes required to encode `x` as a zig-zag encoded signed
/// varint.
pub fn varint_size(x: i64) -> usize {
    let ux = (x as u64) << 1;

    uvarint_size(if x < 0 { !ux } else { ux })
}

/// `write_uvarint` writes 'x' to 'w' encoded as a varint.
pub fn write_uvarint<W>(w: &mut W, mut x: u64) -> io::Result<usize>
    where W: io::Write {

    // A 64-bit varint can be at most 10 bytes long.
    let mut buf = [0u8; 10];
    let mut idx = 0;

    while x > 0x7F {
        buf[idx] = 0x80 | (x & 0x7F) as u8;
        x = x >> 7;
        idx += 1;
    }

    buf[idx] = x as u8;

    try!(w.write_all(&buf[idx..]));

    Ok((idx + 1) as usize)
}

/// `write_varint` writes 'x' to 'w' as a zig-zag encoded signed varint.
pub fn write_varint<W>(w: &mut W, x: i64) -> io::Result<usize>
    where W: io::Write {

    let ux = (x as u64) << 1;

    write_uvarint(w, if x < 0 { !ux } else { ux })
}

/// `write_u8` writes `x` to `w` as a single byte.
pub fn write_u8<W>(w: &mut W, x: u8) -> io::Result<usize>
    where W: io::Write {

    let buf = [x];
    try!(w.write_all(&buf));
    Ok(1)
}

/// `write_le_u16` writes `x` to `w` as 2 bytes in little-endian byte order.
pub fn write_le_u16<W>(w: &mut W, x: u16) -> io::Result<usize>
    where W: io::Write {

    let buf = [
        ((x >> 0) & 0xFF) as u8,
        ((x >> 8) & 0xFF) as u8,
    ];

    try!(w.write_all(&buf));
    Ok(2)
}

/// `write_le_u32` writes `x` to `w` as 4 bytes in little-endian byte order.
pub fn write_le_u32<W>(w: &mut W, x: u32) -> io::Result<usize>
    where W: io::Write {

    let buf = [
        ((x >>  0) & 0xFF) as u8,
        ((x >>  8) & 0xFF) as u8,
        ((x >> 16) & 0xFF) as u8,
        ((x >> 24) & 0xFF) as u8,
    ];

    try!(w.write_all(&buf));
    Ok(4)
}

/// `write_le_u64` writes `x` to `w` as 8 bytes in little-endian byte order.
pub fn write_le_u64<W>(w: &mut W, x: u64) -> io::Result<usize>
    where W: io::Write {

    let buf = [
        ((x >>  0) & 0xFF) as u8,
        ((x >>  8) & 0xFF) as u8,
        ((x >> 16) & 0xFF) as u8,
        ((x >> 24) & 0xFF) as u8,
        ((x >> 32) & 0xFF) as u8,
        ((x >> 40) & 0xFF) as u8,
        ((x >> 48) & 0xFF) as u8,
        ((x >> 56) & 0xFF) as u8,
    ];

    try!(w.write_all(&buf));
    Ok(8)
}

/// `write_i8` writes `x` to `w` as a single, 2's complement encoded byte.
pub fn write_i8<W>(w: &mut W, x: i8) -> io::Result<usize>
    where W: io::Write {

    write_u8(w, x as u8)
}

/// `write_le_i16` writes `x` to `w` as 2 bytes, 2's complement encoded in little-endian byte order.
pub fn write_le_i16<W>(w: &mut W, x: i16) -> io::Result<usize>
    where W: io::Write {

    write_le_u16(w, x as u16)
}

/// `write_le_i32` writes `x` to `w` as 4 bytes, 2's complement encoded in little-endian byte order.
pub fn write_le_i32<W>(w: &mut W, x: i32) -> io::Result<usize>
    where W: io::Write {

    write_le_u32(w, x as u32)
}

/// `write_le_i64` writes `x` to `w` as 8 bytes, 2's complement encoded in little-endian byte order.
pub fn write_le_i64<W>(w: &mut W, x: i64) -> io::Result<usize>
    where W: io::Write {

    write_le_u64(w, x as u64)
}

/// `write_le_f32` writes `x` to `w` as 4 bytes, ieee-754 binary32 encoded in little-endian byte
/// order.
pub fn write_le_f32<W>(w: &mut W, x: f32) -> io::Result<usize>
    where W: io::Write {

    write_le_u32(w, unsafe { mem::transmute(x) })
}

/// `write_le_f64` writes `x` to `w` as 8 bytes, ieee-754 binary64 encoded in little-endian byte
/// order.
pub fn write_le_f64<W>(w: &mut W, x: f64) -> io::Result<usize>
    where W: io::Write {

    write_le_u64(w, unsafe { mem::transmute(x) })
}
