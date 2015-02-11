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

use encoding::Quantifier::*;

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

impl RecordEncoding {
	fn sort_fields(&mut self) {
		self.req_fields.sort_by(|a, b| a.id.cmp(&b.id));
		self.opt_rep_fields.sort_by(|a, b| a.id.cmp(&b.id));
	}
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

impl CompleteEncoding {
	pub fn sort_fields(&mut self) {
		self.target.sort_fields();

		for dep in self.depends.iter_mut() {
			dep.sort_fields();
		}
	}
}

// These are indices into COMPLETE_ENC.depends, below. See docs for that field on the
// CompleteEncoding type.
const FIELD_ENCODING_TYP:  usize = 0 + (Type::FirstUnused as usize);
const RECORD_ENCODING_TYP: usize = 1 + (Type::FirstUnused as usize);

lazy_static! {

	// I apologize in advance for the confusing-ness of this comment.
	//
	// Encodings for records are themselves encoded, so we need to solve the chicken/egg problem in
	// order to be able to interpret the encodings of encodings :3.
	//
	// We do this by providing a pre-decoded encoding for encodings. That's what this lovely
	// structure is.
	pub static ref COMPLETE_ENC: CompleteEncoding = CompleteEncoding {
		target: RecordEncoding {
			name: "CompleteEncoding".to_string(),
			req_fields: vec![

				FieldEncoding {
					id:     1,
					name:   "target".to_string(),
					quant:  Required,
					typ:    RECORD_ENCODING_TYP,
					bounds: None
				},
			],

			opt_rep_fields: vec![

				FieldEncoding {
					id:     2,
					quant:  Repeated,
					name:   "depends".to_string(),
					typ:    RECORD_ENCODING_TYP,
					bounds: None
				},
			],
		},

		depends: vec![

			RecordEncoding {
				name: "FieldEncoding".to_string(),
				req_fields: vec![

					FieldEncoding {
						id:     1,
						name:   "id".to_string(),
						quant:  Required,
						typ:    Type::UInt64 as usize,
						bounds: None
					},

					FieldEncoding {
						id:     2,
						name:   "name".to_string(),
						quant:  Required,
						typ:    Type::String as usize,
						bounds: None
					},

					FieldEncoding {
						id:     3,
						name:   "quant".to_string(),
						quant:  Required,
						typ:    Type::Enum as usize,
						bounds: None
					},

					FieldEncoding {
						id:     4,
						name:   "typ".to_string(),
						quant:  Required,
						typ:    Type::Enum as usize,
						bounds: None
					},

					FieldEncoding {
						id:     5,
						name:   "bounds".to_string(),
						quant:  Required,
						typ:    Type::UInt64 as usize,
						bounds: None
					},
				],
				opt_rep_fields: vec![]
			},

			RecordEncoding {
				name: "RecordEncoding".to_string(),
				req_fields: vec![
					FieldEncoding {
						id:     1,
						name:   "name".to_string(),
						quant:  Required,
						typ:    Type::String as usize,
						bounds: None
					},
				],

				opt_rep_fields: vec![
					FieldEncoding {
						id:     2,
						name:   "req_fields".to_string(),
						quant:  Repeated,
						typ:    FIELD_ENCODING_TYP,
						bounds: None
					},

					FieldEncoding {
						id:     3,
						name:   "opt_rep_fields".to_string(),
						quant:  Repeated,
						typ:    FIELD_ENCODING_TYP,
						bounds: None
					},
				],
			},
		],
	};
}
