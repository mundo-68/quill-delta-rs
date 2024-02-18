// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::attributes::{compose, transform};
use crate::delta::Delta;
use crate::error::Error;
use crate::iterator::DeltaIterator;
use crate::operations::{DeltaOperation, OpType};

pub trait OpTransform {
    /// # Errors
    ///
    /// Returns a Delta that is equivalent to applying the operations of own Delta,
    /// followed by another Delta.
    ///
    /// `other` - Delta to compose
    fn compose(&self, other: &Delta) -> Result<Delta, Error>;

    /// # Errors
    /// Transform given Delta against own operations.
    /// `other` - Delta to transform
    /// `priority` - Boolean used to break ties. If `true`, then `this` takes priority over `other`, that is, its
    /// actions are considered to happened "first".
    fn transform(&self, other: &Delta, priority: bool) -> Result<Delta, Error>;

    /// # Errors
    ///
    /// Transform an index against the quill delta.
    /// Useful for representing cursor/selection positions.
    /// `index` - index to transform
    fn transform_position(&self, index: usize, priority: bool) -> Result<usize, Error>;
}

impl OpTransform for Delta {
    fn compose(&self, other: &Delta) -> Result<Delta, Error> {
        let this_iter = &DeltaIterator::new(self);
        let other_iter = &DeltaIterator::new(other);
        let mut delta = Delta::default();

        //Define closure to handle stuff on the first retain sequence
        let mut handle_retain = |first_other: &DeltaOperation| {
            let mut first_left = first_other.op_len(); //we know here it is a "Retain"
            while this_iter.peek_type() == OpType::Insert && this_iter.peek_len() < first_left {
                first_left -= this_iter.peek_len();
                let t = this_iter.next_len(usize::MAX);
                delta.push(t);
            }
            if first_other.op_len() - first_left > 0 {
                other_iter.next_len(first_other.op_len() - first_left);
            }
        };

        let first_other = other_iter.peek();
        if let Some(val) = first_other {
            if val.op_type() == OpType::Retain {
                handle_retain(val);
            }
        }

        while this_iter.has_next() || other_iter.has_next() {
            if other_iter.peek_type() == OpType::Insert {
                delta.push(other_iter.next_len(0));
            } else if this_iter.peek_type() == OpType::Delete {
                delta.push(this_iter.next_len(0));
            } else {
                let v = [this_iter.peek_len(), other_iter.peek_len()];
                let Some(val) = v.iter().min() else {
                    return Err(Error::EmptyVectorMinOp);
                };
                let l = *val;
                let this_op = this_iter.next_len(l);
                let other_op = other_iter.next_len(l);
                if other_op.op_type() == OpType::Retain {
                    let mut new_op: DeltaOperation = if this_op.op_type() == OpType::Retain {
                        DeltaOperation::retain(l)
                    } else {
                        DeltaOperation::insert(this_op.insert_value().clone())
                    };
                    // Preserve null when composing with a retain, otherwise remove it for inserts
                    let attr = compose(
                        &this_op.attributes,
                        &other_op.attributes,
                        this_op.op_type() == OpType::Retain,
                    );
                    new_op.set_attributes(attr);
                    delta.push(new_op);
                    // Optimization if rest of other is just retain
                    if !other_iter.has_next() {
                        let Some(d_last) = delta.last() else {
                            return Err(Error::EmptyVectorLastOp);
                        };
                        let Some(s_last) = self.last() else {
                            return Err(Error::EmptyVectorLastOp);
                        };
                        if d_last.is_equal(s_last) {
                            let rest = this_iter.rest();
                            return Ok(delta.append_delta_operation(rest).chop().to_owned());
                        }
                    }

                    // Other op should be delete, we could be an insert or retain
                    // Insert + delete cancels out
                } else if other_op.op_type() == OpType::Delete
                    && this_op.op_type() == OpType::Retain
                {
                    delta.push(other_op.clone());
                }
            }
        }
        Ok(delta.chop().to_owned())
    }

    fn transform(&self, other: &Delta, priority: bool) -> Result<Delta, Error> {
        let this_iter = DeltaIterator::new(self);
        let other_iter = DeltaIterator::new(other);
        let mut delta = Delta::default();
        while this_iter.has_next() || other_iter.has_next() {
            if this_iter.peek_type() == OpType::Insert
                && (priority || other_iter.peek_type() != OpType::Insert)
            {
                delta.retain(this_iter.next_len(0).op_len());
            } else if other_iter.peek_type() == OpType::Insert {
                delta.push(other_iter.next_len(0));
            } else {
                let v = [this_iter.peek_len(), other_iter.peek_len()];
                let Some(val) = v.iter().min() else {
                    return Err(Error::EmptyVectorMinOp);
                };
                let l = *val;
                let this_op = this_iter.next_len(l);
                let other_op = other_iter.next_len(l);
                if this_op.op_type() == OpType::Delete {
                    continue;
                } else if other_op.op_type() == OpType::Delete {
                    delta.push(other_op.clone());
                } else {
                    // We retain either their retain or insert
                    delta.retain_attr(
                        l,
                        transform(&this_op.attributes, &other_op.attributes, priority),
                    );
                }
            }
        }

        Ok(delta.chop().to_owned())
    }

    fn transform_position(&self, mut index: usize, priority: bool) -> Result<usize, Error> {
        let this_iter = DeltaIterator::new(self);
        let mut offset: usize = 0;
        while this_iter.has_next() && offset <= index {
            let l = this_iter.peek_len();
            let next_type = this_iter.peek_type();
            this_iter.next_len(0);
            if next_type == OpType::Delete {
                let v = [l, index - offset];
                let Some(val) = v.iter().min() else {
                    return Err(Error::EmptyVectorMinOp);
                };
                index -= *val;
                continue;
            } else if next_type == OpType::Insert && (offset < index || !priority) {
                index += l;
            }
            offset += l;
        }
        Ok(index)
    }
}
