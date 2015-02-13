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

pub struct TakeWhileInclusive<I, P>
    where I: Iterator,
          P: Fn(&<I as Iterator>::Item) -> bool {

    inner: I,
    pred:  P,
    done:  bool,
}

impl<I, P> Iterator for TakeWhileInclusive<I, P>
    where I: Iterator,
          P: Fn(&<I as Iterator>::Item) -> bool {

    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        if self.done {
            None
        } else {
            match self.inner.next() {
                Some(x) => {
                    self.done = !(self.pred)(&x);
                    Some(x)
                },
                None => None,
            }
        }
    }
}

/// `RexIterExt` provides methods on top of iterators (which feel generally useful).
pub trait RexIterExt: Iterator + Sized {

    /// `take_while_incl` returns an iterator which will return the elements of an underlying
    /// iterator until `pred` fails (or the iterator runs out). `take_while_incl` differs from
    /// `take_while` in that it *also returns the element for which `pred` failed*.
    fn take_while_incl<P>(self, pred: P) -> TakeWhileInclusive<Self, P>
        where P: Fn(&<Self as Iterator>::Item) -> bool {

        TakeWhileInclusive {
            inner: self,
            pred:  pred,
            done:  false,
        }
    }
}

impl<I> RexIterExt for I where I: Iterator {}
