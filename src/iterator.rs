// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.


use crate::operations::{DeltaOperation, OpType};
use std::cell::Cell;
use std::option::Option;

/// # DeltaIterator
///
/// Iterator iterating over the content IN the DeltaOperations.
///
/// Hence we do not only iterate over the Objects of type DeltaOperation,
/// but also inside. There are 2 indexes:
///  - index pointing to a DeltaOperation;
///  - offset pointing to a position inside the DeltaOperation.
///
/// For input values, the index may have values
///  - 0 --> first
///  - any usize --> some value in the array
///  - usize::MAX --> end of the list reached
#[allow(clippy::module_name_repetitions)]
pub struct DeltaIterator<'a> {
    ops: &'a Vec<DeltaOperation>, //private list of elements to iterate over
    index: Cell<usize>,           //private index in the vector
    offset: Cell<usize>, //private Position in the string in the DeltaOperation (in case operation == "Insert")
}

impl<'a> DeltaIterator<'a> {
    pub fn new(ops: &'a Vec<DeltaOperation>) -> Self {
        DeltaIterator {
            ops,
            index: Cell::new(0),
            offset: Cell::new(0),
        }
    }

    pub fn has_next(&self) -> bool {
        self.peek_len() < usize::MAX
    }

    /// # peek()
    ///
    /// Returns the delta operation that is next in line to be processed.
    /// But does NOT advance to the next operation.
    ///
    /// # Panics
    /// when internal index offset or index values are wrong
    // Returns the next DeltaOperation
    pub fn peek(&self) -> Option<&DeltaOperation> {
        if self.ops.len() > self.index.get() {
            return Some(self.ops.get(self.index.get()).unwrap());
        }
        None
    }

    /// # next()
    ///
    /// Returns the next operation, and advances the index to the
    /// next operation.
    ///
    /// # Panics
    /// when internal index offset or index values are wrong
    ///
    pub fn next(&self) -> Option<&DeltaOperation> {
        if self.ops.len() > self.index.get() {
            let ret = Some(self.ops.get(self.index.get()).unwrap());
            self.index.set(self.index.get() + 1);
            ret
        } else {
            None
        }
    }

    /// # peek_len()
    ///
    /// Assuming we are on an offset o in an DeltaOperation on Delta operation index i.<br>
    /// We return the remaining length of the Delta operation we point to:
    ///     `op[i].len - offset`
    ///
    /// It should never return 0 if our index is being managed correctly
    ///
    /// # Panics
    /// when internal index offset or index values are wrong
    pub fn peek_len(&self) -> usize {
        if self.ops.len() > self.index.get() {
            self.ops.get(self.index.get()).unwrap().op_len() - self.offset.get()
        } else {
            usize::MAX
        }
    }

    /// # peek_type()
    ///
    /// Returns the `OpType` of the next operation without advancing the index.
    /// # Panics
    /// when internal index offset or index values are wrong
    ///
    // Returns the type of the next DeltaOperation
    pub fn peek_type(&self) -> OpType {
        if self.index.get() < self.ops.len() {
            self.ops.get(self.index.get()).unwrap().op_type()
        } else {
            OpType::Retain
        }
    }

    /// # next_len()
    ///
    /// Returns the next DeltaOperation or a slice thereof
    /// depending on the length of the input parameter len:
    ///
    ///  - If len == 0 the next operation is returned
    ///  - If len > 0 the next operation is returned, or a slice
    ///  - If len takes us past the current DeltaOperation Length, we get the remainder of the DeltaOperation
    ///
    /// # Panics
    /// when internal index offset or index values are wrong
    pub fn next_len(&self, len: usize) -> DeltaOperation {
        let mut length = len;
        if length == 0 {
            length = usize::MAX;
        }

        if self.index.get() < self.ops.len() {
            let offset = self.offset.get();
            let index = self.index.get();
            let next_op = self.ops.get(index).unwrap();

            //Determining the slice we need to take
            let op_length = next_op.op_len();
            let mut act_len = op_length - offset;

            //Updating index for next step
            if length >= act_len {
                //return full DeltaOperation or its remainder
                self.index.set(index + 1);
                self.offset.set(0);
            } else {
                //return slice of the current delta operation
                act_len = length;
                self.offset.set(offset + act_len);
            }

            //returning resulting operation: delete, retain, insert
            match next_op.op_type() {
                OpType::Delete => {
                    let op = DeltaOperation::delete(act_len);
                    return op;
                }
                OpType::Retain => {
                    let mut op = DeltaOperation::retain(act_len);
                    op.set_attributes(next_op.attributes.clone());
                    return op;
                }
                OpType::Insert => {
                    if next_op.is_string() {
                        let s = next_op.string_val().unwrap();
                        let mut op =
                            DeltaOperation::insert(s[offset..offset + act_len].to_string());
                        op.set_attributes(next_op.attributes.clone());
                        return op;
                    }
                    assert_eq!(offset, 0);
                    assert_eq!(act_len, 1);
                    return next_op.clone();
                }
            }
        }
        DeltaOperation::retain(usize::MAX)
    }

    /// # rest()
    ///
    /// Returns the remainder of the operations stack
    /// to which the iterator points
    ///
    /// Leaves the current values for offset and index unchanged
    pub fn rest(&self) -> Vec<DeltaOperation> {
        if !self.has_next() {
            return Vec::new();
        } else if self.offset.get() == 0 {
            return self.ops[self.index.get()..].to_vec();
        }

        // finish fetching the last bit if we are pointing into the middle of an insert operation
        let mut ret = Vec::new();
        ret.push(self.next_len(usize::MAX));
        if let Some(d) = self.ops.get(self.index.get()..) {
            ret.append(&mut d.to_vec());
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::Attributes;
    use crate::delta::Delta;

    #[test]
    fn delta_len_passes() {
        let o = DeltaOperation::delete(5);
        assert_eq!(o.op_len(), 5);

        let o = DeltaOperation::retain(2);
        assert_eq!(o.op_len(), 2);

        let o = DeltaOperation::insert("text");
        assert_eq!(o.op_len(), 4);

        let o = DeltaOperation::insert(2);
        assert_eq!(o.op_len(), 1);
    }

    fn get_delta() -> Delta {
        let mut attr = Attributes::default();
        attr.insert("bold", true);

        let mut img = Attributes::default();
        img.insert("srt", "http://quilljs.com/");

        let mut delta = Delta::default();
        delta.insert_attr("Hello".to_string(), attr);
        delta.retain(3);
        delta.insert_attr(2, img);
        delta.delete(4);
        delta
    }

    #[test]
    fn delta_iter_has_next_passes() {
        let delta = get_delta();
        let iter = DeltaIterator::new(&delta);
        assert!(iter.has_next());
    }

    #[test]
    fn delta_iter_has_not_next_passes() {
        let delta = Delta::default();
        let iter = DeltaIterator::new(&delta);
        assert!(!iter.has_next());
    }

    #[test]
    fn delta_iter_peek_length_offset_null_passes() {
        let delta = get_delta();
        let iter = DeltaIterator::new(&delta);
        assert_eq!(iter.peek_len(), 5);
        iter.next_len(0);
        assert_eq!(iter.peek_len(), 3);
        iter.next_len(0);
        assert_eq!(iter.peek_len(), 1);
        iter.next_len(0);
        assert_eq!(iter.peek_len(), 4);
        iter.next_len(0);
    }

    #[test]
    fn delta_iter_peek_length_offset_gt_null_passes() {
        let delta = get_delta();
        let iter = DeltaIterator::new(&delta);
        iter.next_len(2);
        assert_eq!(iter.peek_len(), 5 - 2);
    }

    #[test]
    fn delta_iter_no_ops_left_passes() {
        let delta = Delta::default();
        let iter = DeltaIterator::new(&delta);
        assert_eq!(iter.peek_len(), usize::MAX);
    }

    #[test]
    fn delta_iter_peek_type_passes() {
        let delta = get_delta();
        let iter = DeltaIterator::new(&delta);
        assert_eq!(iter.peek_type(), OpType::Insert);
        iter.next_len(0);
        assert_eq!(iter.peek_type(), OpType::Retain);
        iter.next_len(0);
        assert_eq!(iter.peek_type(), OpType::Insert);
        iter.next_len(0);
        assert_eq!(iter.peek_type(), OpType::Delete);
        iter.next_len(0);
        assert_eq!(iter.peek_type(), OpType::Retain);
        iter.next_len(0);
    }

    #[test]
    fn delta_iter_next_passes() {
        let delta = get_delta();
        let iter = DeltaIterator::new(&delta);
        for i in 0..delta.len() {
            assert_eq!(iter.next_len(0), *delta.get(i).unwrap());
        }
        assert_eq!(iter.next_len(0), DeltaOperation::retain(usize::MAX));
        assert_eq!(iter.next_len(4), DeltaOperation::retain(usize::MAX));
        assert_eq!(iter.next_len(0), DeltaOperation::retain(usize::MAX));
    }

    #[test]
    fn delta_iter_next_length_passes() {
        let mut attr = Attributes::default();
        attr.insert("bold".to_string(), true);

        let delta = get_delta();
        let iter = DeltaIterator::new(&delta);
        let nxt = iter.next_len(2);
        let mut expect = DeltaOperation::insert("He");
        expect.set_attributes(attr);
        assert_eq!(nxt, expect);

        let nxt = iter.next_len(10);
        let mut attr = Attributes::default();
        attr.insert("bold".to_string(), true);
        let mut expect = DeltaOperation::insert("llo");
        expect.set_attributes(attr);
        assert_eq!(nxt, expect);

        let nxt = iter.next_len(1);
        let expect = DeltaOperation::retain(1);
        assert_eq!(nxt, expect);

        let nxt = iter.next_len(2);
        let expect = DeltaOperation::retain(2);
        assert_eq!(nxt, expect);
    }

    #[test]
    fn delta_iter_rest_1_passes() {
        let mut attr = Attributes::default();
        attr.insert("bold".to_string(), true);

        let mut img = Attributes::default();
        img.insert("srt".to_string(), "http://quilljs.com/");

        let mut expect = Delta::default();
        expect.insert_attr("llo", attr);
        expect.retain(3);
        expect.insert_attr(2, img);
        expect.delete(4);

        let delta = get_delta();
        let iter = DeltaIterator::new(&delta);
        iter.next_len(2);

        assert_eq!(iter.rest(), expect.get_ops());
    }

    #[test]
    fn delta_iter_rest_2_passes() {
        let mut img = Attributes::default();
        img.insert("srt".to_string(), "http://quilljs.com/");

        let mut expect = Delta::default();
        expect.retain(3);
        expect.insert_attr(2, img);
        expect.delete(4);

        let delta = get_delta();
        let iter = DeltaIterator::new(&delta);
        iter.next_len(2);
        iter.next_len(3);

        assert_eq!(iter.rest(), expect.get_ops());

        iter.next_len(3);
        iter.next_len(2);
        iter.next_len(4);
        let tv: Vec<DeltaOperation> = Vec::new();
        assert_eq!(iter.rest(), tv);
    }
}
