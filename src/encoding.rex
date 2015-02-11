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

package rex;

enum Type {
	// Fixed, 1-byte, 2's complement
	Int8 = 0

	// Fixed, 2-bytes, little-endian, 2's complement
	Int16 = 1

	// Varint, zig-zag encoded
	Int32 = 2
	Int64 = 3

	// Fixed, 1-byte
	UInt8 = 4

	// Fixed, 2-bytes, little-endian
	UInt16 = 5

	// Varint
	UInt32 = 6
	UInt64 = 7

	// Fixed, 4-bytes, little-endian
	Fixed32 = 8

	// Fixed, 8-bytes, little-endian
	Fixed64 = 9

	// Fixed, 4-bytes, little-endian
	Float32 = 10

	// Fixed, 8-bytes, little-endian
	Float64 = 11

	// Varint length, followed by raw bytes.
	Bytes = 12

	// Varint length, followed by raw bytes, utf-8 encoded.
	String = 13

	// Fixed, 1-byte. 0xFF is true, 0x00 is false.
	Bool = 14

	// Varint
	Enum = 15

	// Each record type 
	FirstUnused = 16
}

enum Quantifier {
	Required = 0,
	Optional = 1,
	Repeated = 2,
}

record FieldEncoding {
	// Integer id of this field within its containing record.
	1 id: uint64

	// Name of this field in the .rex file, not used in the encoding.
	2 name: string

	// Is this field Required, Optional (opt), or Repeated (rep)?
	3 quant: Quantifier

	// Type of this field
	4 typ: Type

    // The bounds field is the product of all bounds in an array field. So for example, the field
    //
    //     1 matrix : [3][3]float32
    //
    // would have a bounds field of 9.
    //
    // The bounds field is not present for non-array types.
	5 bounds: opt uint64
}

record RecordEncoding {
	// Name of the record type in the .rex file, not used in the encoding.
	1 name: string

	// Required fields of this record type, sorted by id.
	2 req_fields: rep FieldEncoding

	// Optional and repeated fields of this record type, sorted by id.
	3 opt_rep_fields: rep FieldEncoding
}

// A CompleteEncoding provides all of the information necessary to parse a particular record type
// (and every record type that it can contain).
record CompleteEncoding {
	// The record type that this CompleteEncoding describes
	1 target: RecordEncoding

	// Encodings for all dependencies of target. If a field has a type (t >= Type::FirstUnused),
	// then a RecordEncoding for that type is at depends[t - Type::FirstUnused].
	2 depends: rep RecordEncoding
}
