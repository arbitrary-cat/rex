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

#![feature(io)]
#![feature(core)]
#![deny(missing_docs)]

//! `rex` is a record encoding format designed for use in games.

#[macro_use]
extern crate lazy_static;

/// The `encoding` module defines the structures which are used to describe record encodings. The
/// data structures described in this module drive `Encoder`s and `Decoder`s.
pub mod encoding;

/// The `encoder` module defines the `Encoder` and related types.
pub mod encoder;

/// The `primitive` module provides helper methods for working with primitive types in rex, and
/// defines the `Primitive` type which provides rust representations for each of the primitive
/// types.
pub mod primitive;
