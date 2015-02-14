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

// Allow dead code to silence warnings until things stabilize.
#![allow(dead_code)]

use std::error::FromError;
use std::io;
use std::io::ReadExt;
use std::mem;
use std::result;
use std::string::FromUtf8Error;

use encoding::{CompleteEncoding, RecordEncoding, FieldEncoding, Type, FieldID};
use primitive::Primitive;

use iter::{RexIterExt, ResultIterExt};

/// `Error` is used to report errors that occur during the decoding process.
pub enum Error {
    /// `EOF` indicates that record source ended before it could be fully decoded.
    EOF,

    /// `FieldTypeMismatch` indicates a disagreement between the `Decoder` and the `Decodable`
    /// about what the type of a field is. This probably indicates one of two things:
    ///
    /// 1. The wrong `CompleteEncoding` is being used to decode this type.
    /// 2. The `Decodable` implementation is not consistent with the record definition.
    FieldTypeMismatch,

    /// `EncodingInvalid` indicates that there is an inconsistency in the `CompleteEncoding` being
    /// used. A probable fix is to regenerate the encoding (or inspect the encoding compiler for
    /// errors).
    EncodingInvalid,

    /// `BadBool` indicates that a bool was read off the wire as a value other than `0xFF` or
    /// `0x00`, indicating that the decoder is incorrect in expecting it to be a bool.
    BadBool,

    /// `Utf8Error` is used to pass through `std::str::FromUtf8Error`s.
    Utf8Error(FromUtf8Error),

    /// `IoError` is used to pass through `std::io` errors.
    IoError(io::Error),
}

impl FromError<io::Error> for Error {
    fn from_error(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl FromError<FromUtf8Error> for Error {
    fn from_error(err: FromUtf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

/// The `Decodable` trait allows an object to be decoded from a rex record.
pub trait Decodable {
    /// `set_primitive` sets the value of a single element of a field with primitive type.
    fn set_primitive(&mut self, id: FieldID, idx: usize, prim: Primitive) -> Result<(), Error>;

    /// `decode_record` is a request to call `d.decode` on a record field.
    fn decode_record<'x, R>(&mut self, d: Decoder<'x, R>, id: FieldID, idx: usize) -> Result<(), Error>
        where R: io::Read + 'x;

    /// `alloc_field` requests that the receiver allocate space for an optional or repeated field.
    /// The boolean return value can be `false` to indicate that the receiver is uninterested in
    /// this field.
    fn alloc_field(&mut self, id: FieldID, count: usize) -> Result<bool, Error>;
}

/// A `Decoder` is a struct that knows how to decode a particular record field. `Decoder`s should
/// only be used where they are passed to the `decode_record` method of a `Decodable`.
pub struct Decoder<'x, R: io::Read + 'x> {
    // `r` is the reader that the record is being read from.
    r: &'x mut R,

    // `rec` is the record type that this `Decoder` knows how to decode.
    rec: &'x RecordEncoding,

    // `deps` is a (full) slice of the `depends` field of the `CompleteEncoding` that `rec` is a
    // member of.
    deps: &'x [RecordEncoding],
}

pub fn decode_from<'x, R, D>(enc: &'x CompleteEncoding, r: &'x mut R, d: &'x mut D) -> Result<(), Error>
    where R: io::Read + 'x,
          D: Decodable {

    let mut dec = Decoder {
        r:    r,
        rec:  &enc.target,
        deps: &enc.depends[],
    };

    dec.decode(d)
}

impl<'x, R> Decoder<'x, R>
    where R: io::Read + 'x {

    /// `decode` decodes the next record on the wire into `d`.
    pub fn decode<D>(&mut self, d: &mut D) -> Result<(), Error>
        where D: Decodable {

        use encoding::Quantifier::*;

        for req_field in self.rec.req_fields.iter() {
            try!(self.decode_required(d, req_field));
        }

        let mut opt_rep_itr = self.rec.opt_rep_fields.iter();
        let mut next_field  = opt_rep_itr.next();
        let mut next_id     = FieldID(try!(read_uvarint(self.r)));

        while next_id != FieldID(0) {
            match next_field {
                Some(field) => if field.id < next_id {
                    next_field = opt_rep_itr.next();
                } else if field.id > next_id {
                    try!(self.skip_field());
                    next_id = FieldID(try!(read_uvarint(self.r)));
                } else {
                    match field.quant {
                        Required => return Err(Error::EncodingInvalid),
                        Optional => try!(self.decode_optional(d, field)),
                        Repeated => try!(self.decode_repeated(d, field)),
                    }
                    next_field = opt_rep_itr.next();
                    next_id    = FieldID(try!(read_uvarint(self.r)));
                },
                None => next_id = FieldID(try!(read_uvarint(self.r))),
            }
        }

        Ok(())
    }

    fn skip_field(&mut self) -> Result<(), Error> {
        let len = try!(read_uvarint(self.r)) as usize;
        result::fold(self.r.bytes().take_or_err(len, Error::EOF), (), |(), _| ())
    }

    fn child(&mut self, index: usize) -> Result<Decoder<R>, Error> {
        if index < self.deps.len() {
            Ok( Decoder {
                r:      self.r,
                rec:    &self.deps[index],
                deps:   self.deps,
            })
        } else {
            Err(Error::EncodingInvalid)
        }
    }

    fn decode_required<D>(&mut self, d: &mut D, f: &FieldEncoding) -> Result<(), Error>
        where D: Decodable {

        self.decode_array(d, f, 0)
    }

    fn decode_optional<D>(&mut self, d: &mut D, f: &FieldEncoding) -> Result<(), Error>
        where D: Decodable {

        // Discard the byte-size prefix. We don't need it.
        try!(read_uvarint(self.r));

        self.decode_array(d, f, 0)
    }

    fn decode_repeated<D>(&mut self, d: &mut D, f: &FieldEncoding) -> Result<(), Error>
        where D: Decodable {

        // Discard the byte-size prefix. We don't need it.
        try!(read_uvarint(self.r));

        let len = try!(read_uvarint(self.r)) as usize;

        for idx in 0..len {
            try!(self.decode_array(d, f, idx));
        }

        Ok(())
    }



    fn decode_array<D>(&mut self, d: &mut D, f: &FieldEncoding, idx: usize) -> Result<(), Error>
        where D: Decodable {

        if let Some(max) = f.bounds {
            for arr_index in 0..max {
                try!(self.decode_field(d, f, idx*max + arr_index));
            }
        } else {
            try!(self.decode_field(d, f, idx));
        }

        Ok(())
    }

    fn decode_field<D>(&mut self, d: &mut D, f: &FieldEncoding, idx: usize) -> Result<(), Error>
        where D: Decodable {

        let prim = match f.typ {
            Type::UInt8  => Primitive::UInt8(try!(read_u8(self.r))),
            Type::UInt16 => Primitive::UInt16(try!(read_le_u16(self.r))),
            Type::UInt32 => Primitive::UInt32(try!(read_uvarint(self.r)) as u32),
            Type::UInt64 => Primitive::UInt64(try!(read_uvarint(self.r))),

            Type::Int8  => Primitive::Int8(try!(read_i8(self.r))),
            Type::Int16 => Primitive::Int16(try!(read_le_i16(self.r))),
            Type::Int32 => Primitive::Int32(try!(read_varint(self.r)) as i32),
            Type::Int64 => Primitive::Int64(try!(read_varint(self.r))),

            Type::Fixed32 => Primitive::Fixed32(try!(read_le_u32(self.r))),
            Type::Fixed64 => Primitive::Fixed64(try!(read_le_u64(self.r))),

            Type::Float32 => Primitive::Float32(try!(read_le_f32(self.r))),
            Type::Float64 => Primitive::Float64(try!(read_le_f64(self.r))),

            Type::Bool => Primitive::Bool(match try!(read_u8(self.r)) {
                0xFF => true,
                0x00 => false,
                _    => return Err(Error::BadBool),
            }),

            Type::Bytes => Primitive::Bytes({
                let len = try!(read_uvarint(self.r)) as usize;
                try!(result::fold(
                    self.r.bytes().take_or_err(len, Error::EOF),
                    Vec::with_capacity(len),
                    |mut v, elem| {
                        v.push(elem);
                        v
                }))
            }),

            Type::String => Primitive::String({
                let len = try!(read_uvarint(self.r)) as usize;
                let utf8 = try!(result::fold(
                    self.r.bytes().take_or_err(len, Error::EOF),
                    Vec::with_capacity(len),
                    |mut v, elem| {
                        v.push(elem);
                        v
                }));

                try!(String::from_utf8(utf8))
            }),

            Type::Enum => Primitive::Enum(try!(read_varint(self.r))),

            // Records work a little differently. Create a child decoder and have the `Decodable`
            // run it on its own record field.
            Type::Record{index: dep_index} => {
                let child = try!(self.child(dep_index));
                return d.decode_record(child, f.id, idx);
            },
        };

        d.set_primitive(f.id, idx, prim)
    }
}

/// `read_uvarint` reads a varint encoded `u64` from `r`.
fn read_uvarint<R>(r: &mut R) -> Result<u64, Error>
    where R: io::Read {

    let itr = r.bytes()
        .take_while_incl(|x| match x { &Ok(x) => x >= 0x80, _ => true });

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
