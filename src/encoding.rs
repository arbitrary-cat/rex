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

// TODO: Remove this when things stabilize.
#![allow(dead_code)]

#[derive(Copy,Eq,PartialEq)]
pub enum Type {
	Int8  = 0,
	Int16 = 1,
	Int32 = 2,
	Int64 = 3,

	UInt8  = 4,
	UInt16 = 5,
	UInt32 = 6,
	UInt64 = 7,
	
	Fixed32 = 8,
	Fixed64 = 9,

	Float32 = 10,
	Float64 = 11,

	Bytes  = 12,
	String = 13,

	Bool = 14,

	Enum = 15,

	FirstUnused = 16,
}

// The Quantifier type gives the multiplicity of a field. A Required field has exactly 1 element, an
// Optional field has 0 or 1 elements, and a Repeated field has 0 or more elements.
#[derive(Debug,Copy,PartialEq,Eq)]
pub enum Quantifier {
	Required = 0,
	Optional = 1,
	Repeated = 2,
}

#[derive(PartialEq,Eq)]
pub struct FieldEncoding {
	// Integer id of this field within its containing record.
	pub id: u64,

	// Name of this field in the .rex file, not used in the encoding.
	pub name: String,

	// Is this field Required, Optional (opt), or Repeated (rep)?
	pub quant: Quantifier,

	// Type of this field.
	pub typ: usize,

    // The bounds field is the product of all bounds in an array field. So for example, the field
    //
    //     1 matrix : [3][3]float32
    //
    // would have a bounds field of 3*3 = 9.
    //
    // The bounds field is not present for non-array types.
	pub bounds: Option<usize>,
}


#[derive(PartialEq,Eq)]
pub struct RecordEncoding {
	// Name of the record type in the .rex file, not used in the encoding.
	pub name: String,

	// Required fields of this record type, sorted by id.
	pub req_fields: Vec<FieldEncoding>,

	// Optional and repeated fields of this record type, sorted by id.
	pub opt_rep_fields: Vec<FieldEncoding>,
}

// A CompleteEncoding provides all of the information necessary to parse a particular record type
// (and every record type that it can contain).
#[derive(PartialEq,Eq)]
pub struct CompleteEncoding {
	// The record type that this CompleteEncoding describes.
	pub target:  RecordEncoding,

	// Encodings for all dependencies of target. If a field has a type (t >= Type::FirstUnused),
	// then a RecordEncoding for that type is at depends[t - Type::FirstUnused].
	pub depends: Vec<RecordEncoding>,
}
