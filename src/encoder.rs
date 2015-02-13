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

#![allow(dead_code)]

use std::io;
use std::error::FromError;

use encoding::{CompleteEncoding, RecordEncoding, FieldEncoding, Type, FieldID};
use primitive::Primitive;

/// `Error` is used to report errors that occur during the encoding process.
pub enum Error {
    /// `EncodingInvalid` indicates that there is an inconsistency in the `CompleteEncoding` being
    /// used. A probable fix is to regenerate the encoding (or inspect the encoding compiler for
    /// errors).
    EncodingInvalid,

    /// `FieldTypeMismatch` indicates a disagreement between the `Encoder` and the `Encodable`
    /// about what the type of a field is. This probably indicates one of two things:
    ///
    /// 1. The wrong `CompleteEncoding` is being used to decode this type.
    /// 2. The `Encodable` implementation is not consistent with the record definition.
    FieldTypeMismatch,

    /// `IoError` allows propogation of i/o errors which are unrelated to the encoding process.
    IoError(io::Error),
}

impl FromError<io::Error> for Error {
    fn from_error(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

/// The `Encodable` trait allows an object to be encoded as a rex record.
pub trait Encodable {
    /// `get_primitive` should return the value of a single element of a field with primitive type.
    fn get_primitive(&self, id: FieldID, idx: usize) -> Result<Primitive, Error>;

    /// `encode_record` is a request to call `e.encode` on a record field.
    fn encode_record<'x>(&self, e: Encoder<'x>, id: FieldID, idx: usize) -> Result<usize, Error>;

    /// `count_field` should return the number of members of an optional or repeated field.
    fn count_field(&self, id: FieldID) -> Result<usize, Error>;
}

/// A `Chunk` gives the offset into an `Encoder`'s `data` buffer at which a byte-size prefix needs
/// to be written in the final output stream.
struct Chunk {
    offset: usize,
    size:   usize,
}

/// An `Encoder` is a struct that knows how to encode a particular record field. `Encoder`s should
/// only be used where they are passed to the `encode_record` method of an `Encodable`.
pub struct Encoder<'x> {
    // `rec` is the encoding for the record type that this `Encoder` knows how to encode.
    rec: &'x RecordEncoding,

    // `deps` is a (full) slice of the `depends` field of the `CompleteEncoding` that `rec` is a
    // member of.
    deps: &'x [RecordEncoding],

    // `data` is the buffer into which the encoded data *other than byte-size prefixes* will be
    // written. It is a staging area, since not all byte-size prefixes can be computed before the
    // encoding is done.
    data: &'x mut Vec<u8>,

    // `chunks` is an unsorted list of byte-size prefixes along with the indices into `data` at
    // which they should be written.
    chunks: &'x mut Vec<Chunk>,
}

impl<'x> Encoder<'x> {
    /// The `encode` method should be called by implementations of the `encode_record` method of
    /// the `Encodable` trait. See that method's documentation for example usage.
    ///
    /// Arguments
    /// ---------
    /// `rec_field` -- An `Encodable` which corresponds to a rex field with record type.
    ///
    /// Return Value
    /// ------------
    /// On success, `encode` will return the number of bytes required to *fully encode*
    /// `rec_field`. This includes the size of byte-size prefixes.
    ///
    /// On error, it will return an `Error`.
    ///
    /// In either case, the caller (an implementation of `Encodable`) should pass the return value
    /// directly through as the return value of `encode_record`.
    pub fn encode<E>(&mut self, e: &mut E) -> Result<usize, Error>
        where E: Encodable {

        use primitive::write_uvarint;

        use encoding::Quantifier::*;

        let mut total = 0;

        for req_field in self.rec.req_fields.iter() {
            total += try!( match req_field.quant {
                Required            => self.encode_required(e, req_field),
                Repeated | Optional => Err(Error::EncodingInvalid),
            })
        }

        for opt_rep_field in self.rec.opt_rep_fields.iter() {
            total += match opt_rep_field.quant {
                Optional => try!(self.encode_optional(e, opt_rep_field)),
                Repeated => try!(self.encode_repeated(e, opt_rep_field)),
                Required => return Err(Error::EncodingInvalid),
            }
        }

        // Write the final 0-id, marking the end of the record.
        total += try!(write_uvarint(self.data, 0));

        Ok(total)
    }

    // Create an encoder with the same `data`/`chunks` fields, but which
    fn child(&mut self, index: usize) -> Encoder {
        Encoder {
            rec:    &self.deps[index],
            deps:   self.deps,
            data:   self.data,
            chunks: self.chunks,
        }
    }

    fn encode_required<E>(&mut self, e: &E, f: &FieldEncoding) -> Result<usize, Error>
        where E: Encodable {

        self.encode_array(e, f, 0)
    }

    fn encode_optional<E>(&mut self, e: &E, f: &FieldEncoding) -> Result<usize, Error>
        where E: Encodable {

        use primitive::uvarint_size;

        let max = try!(e.count_field(f.id));
        if max == 0 {
            return Ok(0);
        }

        let len_data        = try!(self.encode_array(e, f, 0));
        let len_id_prefix   = { let FieldID(id) = f.id; uvarint_size(id as u64) };
        let len_size_prefix = uvarint_size(len_data as u64);

        Ok(len_id_prefix + len_size_prefix + len_data)
    }

    fn encode_repeated<E>(&mut self, e: &E, f: &FieldEncoding) -> Result<usize, Error>
        where E: Encodable {

        use primitive::uvarint_size;

        let max = try!(e.count_field(f.id));
        if max == 0 {
            return Ok(0);
        }

        // Bytes required to encode the data itself
        let mut len_data = 0;

        for index in 0..max {
            len_data += try!(self.encode_array(e, f, index));
        }

        let len_id_prefix     = { let FieldID(id) = f.id; uvarint_size(id as u64) };
        let len_length_prefix = uvarint_size(max as u64);
        let len_size_prefix   = uvarint_size((len_length_prefix + len_data) as u64);

        Ok(len_id_prefix + len_size_prefix + len_length_prefix + len_data)
    }

    fn encode_array<E>(&mut self, e: &E, f: &FieldEncoding, index: usize) -> Result<usize, Error>
        where E: Encodable {
        match f.bounds {
            Some(max) => {
                let mut total = 0;
                for arr_index in 0..max {
                    total += try!(self.encode_field(e, f, index*max + arr_index));
                }
                Ok(total)
            }
            None => self.encode_field(e, f, index),
        }
    }

    fn encode_field<E>(&mut self, e: &E, f: &FieldEncoding, index: usize) -> Result<usize, Error>
        where E: Encodable {

        if let Type::Record{index: child_index} = f.typ {
            e.encode_record(self.child(child_index), f.id, index)
        } else {
            let prim = try!(e.get_primitive(f.id, index));
            if !prim.has_type(f.typ) {
                return Err(Error::FieldTypeMismatch);
            }

            self.encode_primitive(prim)
        }
    }

    fn encode_primitive(&mut self, prim: Primitive) -> Result<usize, Error> {

        use primitive::*;

        Ok( match prim {
            Primitive::UInt8(x)  => try!(write_le_u8(self.data, x)),
            Primitive::UInt16(x) => try!(write_le_u16(self.data, x)),
            Primitive::UInt32(x) => try!(write_uvarint(self.data, x as u64)),
            Primitive::UInt64(x) => try!(write_uvarint(self.data, x)),

            Primitive::Int8(x)  => try!(write_le_i8(self.data, x)),
            Primitive::Int16(x) => try!(write_le_i16(self.data, x)),
            Primitive::Int32(x) => try!(write_varint(self.data, x as i64)),
            Primitive::Int64(x) => try!(write_varint(self.data, x)),

            Primitive::Fixed32(x) => try!(write_le_u32(self.data, x)),
            Primitive::Fixed64(x) => try!(write_le_u64(self.data, x)),

            Primitive::Float32(x) => try!(write_le_f32(self.data, x)),
            Primitive::Float64(x) => try!(write_le_f64(self.data, x)),

            Primitive::Bool(x) => try!(write_le_u8(self.data, if x { 0xFF } else { 0x00 })),

            Primitive::Bytes(x) => {
                try!(write_uvarint(self.data, x.len() as u64));
                try!(io::Write::write_all(self.data, &x));
                x.len()
            }

            Primitive::String(x) => {
                let utf8 = x.as_bytes();
                try!(write_uvarint(self.data, utf8.len() as u64));
                try!(io::Write::write_all(self.data, utf8));
                utf8.len()
            }

            Primitive::Enum(x) => try!(write_varint(self.data, x)),
        })
    }
}
