// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::attributes::Attributes;
pub use crate::document::Document;
use crate::operations::{DeltaOperation, OpType, OpsVal};
use crate::types::ops_kind::OpKind;
use serde_derive::{Deserialize, Serialize};
#[cfg(test)]
use std::fmt::{Display, Formatter};

/// Deltas are a simple, yet expressive format that can be used to describe contents and changes.
/// The format is JSON based, and is human readable, yet easily parsible by machines.
/// Deltas can describe any rich text document, includes all text and formatting information,
/// without the ambiguity and complexity of HTML.
///
/// A Delta is made up of an Array of Operations, which describe changes to a document.
/// They can be an insert, delete or retain. Note operations do not take an index.
/// They always describe the change at the current index. Use retains to "keep" or "skip" certain
/// parts of the document.
///
/// Don’t be confused by its name Delta—Deltas represents both documents and changes to documents.
/// If you think of Deltas as the instructions from going from one document to another,
/// the way Deltas represent a document is by expressing the instructions starting from
/// an empty document.

/// Wrapper to manipulate Delta easily
/// ```
/// extern crate delta;
/// use delta::delta::Delta;
/// use delta::operations::*;
/// use delta::attributes;
///
///     let mut _quill_delta = Delta::default();
///     _quill_delta.retain(2);
///     _quill_delta.insert("Hallo World");
///
///     let _quill_delta:Delta = vec![
///         DeltaOperation::retain(2),
///         DeltaOperation::insert("Hallo World")
///     ].into();
/// ```
///
/// Delta represents a document or a modification of a document as a sequence of
///  insert, delete and retain operations.
///
///  Delta consisting of only "insert" operations is usually referred to as
///  "document delta". When delta includes also "retain" or "delete" operations
///  it is a "change delta".
///
// https://github.com/maximkornilov/types-quill-delta/blob/master/index.d.ts
// https://github.com/quilljs/delta#insert-operation
#[derive(Clone, Default, PartialEq, Debug, Serialize, Deserialize)]
pub struct Delta {
    //#[serde(flatten)]
    ops: Vec<DeltaOperation>,
}

impl Delta {
    pub fn new(ops: Vec<DeltaOperation>) -> Self {
        Delta { ops }
    }

    pub(crate) fn chop(&mut self) -> &mut Delta {
        if !self.ops.is_empty() {
            let Some(last_op) = self.ops.last() else {
                return self;
            };
            if let OpType::Retain = last_op.op_type() {
                if last_op.get_attributes().is_empty() {
                    self.ops.pop();
                }
            }
        }
        self
    }

    pub fn insert<S: Into<OpsVal>>(&mut self, value: S) {
        let op = DeltaOperation::insert(value);
        if op.op_len() == 0 {
            return;
        }
        self.push(op);
    }

    pub fn insert_attr<S: Into<OpsVal>>(&mut self, value: S, attributes: Attributes) {
        let mut op = DeltaOperation::insert(value);
        if op.op_len() == 0 {
            return;
        }
        op.set_attributes(attributes);
        self.push(op);
    }

    pub fn retain(&mut self, length: usize) {
        if length == 0 {
            return;
        }
        self.push(DeltaOperation::retain(length));
    }

    pub fn retain_attr(&mut self, length: usize, attributes: Attributes) {
        if length == 0 {
            return;
        }
        let mut op = DeltaOperation::retain(length);
        op.set_attributes(attributes);
        self.push(op);
    }

    pub fn delete(&mut self, length: usize) {
        if length == 0 {
            return;
        }
        self.push(DeltaOperation::delete(length));
    }

    ///
    ///Private function to add one operation to the end of the vector
    ///
    /// Pushes new operation into this delta.
    ///
    /// Performs compaction by composing [operation] with current tail operation
    /// of this delta, when possible. For instance, if current tail is
    /// `insert('abc')` and pushed operation is `insert('123')` then existing
    /// tail is replaced with `insert('abc123')` - a compound result of the two
    /// operations.
    pub fn push(&mut self, new_op: DeltaOperation) {
        let Some(last_op) = self.ops.pop() else {
            self.ops.push(new_op);
            return;
        };

        // Merge new operations to the existing operations on the stack if possible
        match &new_op.kind {
            OpKind::Insert(_insert) => match last_op.op_type() {
                OpType::Delete => {
                    // order of insert and delete may be swapped without giving the same delta result
                    // we always insert before delete
                    // as a result, repeated insert / delete are nicely collected into 1 operation if possible
                    let Some(tmp) = self.ops.pop() else {
                        self.ops.push(new_op);
                        self.ops.push(last_op);
                        return;
                    };
                    if let Ok(new_s) = new_op.string_val() {
                        if let Ok(last_s) = tmp.string_val() {
                            if new_op.attributes.is_equal(&tmp.attributes) {
                                let op = DeltaOperation::insert_attr(
                                    [last_s, new_s].concat(),
                                    new_op.attributes,
                                );
                                self.ops.push(op);
                                self.ops.push(last_op);
                                return;
                            }
                        }
                    }
                    self.ops.push(tmp);
                    self.ops.push(new_op);
                    self.ops.push(last_op);
                    return;
                }
                OpType::Insert => {
                    if let Ok(last_s) = last_op.string_val() {
                        if let Ok(new_s) = new_op.string_val() {
                            if last_op.attributes.is_equal(&new_op.attributes) {
                                let op = DeltaOperation::insert_attr(
                                    [last_s, new_s].concat(),
                                    last_op.attributes,
                                );
                                self.ops.push(op);
                                return;
                            }
                        }
                    }
                }
                OpType::Retain => {}
            },
            OpKind::Retain(retain) => {
                if last_op.op_type() == OpType::Retain && last_op.attributes == new_op.attributes {
                    let op =
                        DeltaOperation::retain_attr(last_op.op_len() + retain, new_op.attributes);
                    self.ops.push(op);
                    return;
                }
            }
            OpKind::Delete(delete) => {
                if last_op.op_type() == OpType::Delete {
                    let op = DeltaOperation::delete(last_op.op_len() + delete);
                    self.ops.push(op);
                    return;
                }
            }
        }

        self.ops.push(last_op);
        self.ops.push(new_op);
    }

    pub fn append(&mut self, mut delta: Delta) {
        let first = delta.ops.remove(0);
        self.push(first);
        self.ops.append(&mut delta.ops);
    }

    pub(crate) fn append_delta_operation(&mut self, mut other: Vec<DeltaOperation>) -> &mut Delta {
        if !other.is_empty() {
            self.push(other.remove(0)); //merges repeated retain, delete, insert
            self.extend(other); //should have no repetitions
        }
        self
    }

    pub fn get_ops(self) -> Vec<DeltaOperation> {
        self.ops
    }

    pub fn get_ops_ref(&self) -> &Vec<DeltaOperation> {
        &self.ops
    }
}

impl std::ops::Deref for Delta {
    type Target = Vec<DeltaOperation>;
    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}

impl std::ops::DerefMut for Delta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ops
    }
}

impl From<Vec<DeltaOperation>> for Delta {
    fn from(ops: Vec<DeltaOperation>) -> Delta {
        Delta { ops }
    }
}

impl std::iter::FromIterator<DeltaOperation> for Delta {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = DeltaOperation>,
    {
        let res: Vec<_> = iter.into_iter().collect();
        res.into()
    }
}

//Note display is one form is serialization, but we can not read it back.
//Use json serializer instead
#[cfg(test)]
impl Display for Delta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Delta --> [[").ok();
        let mut count = 0;
        for o in &self.ops {
            count += 1;
            writeln!(f, "\t{count}: {o}").ok();
        }
        writeln!(f, "]]")
    }
}

#[cfg(test)]
#[test]
fn helper_chop_test() {
    let mut a = Delta::default();
    a.insert("Test".to_string());
    a.retain(4);

    let mut expected = Delta::default();
    expected.insert("Test".to_string());

    a.chop();
    assert_eq!(a, expected);
}

#[test]
fn helper_insert_chop_test() {
    let mut a = Delta::default();
    a.insert("Test");

    let mut expected = Delta::default();
    expected.insert("Test");

    a.chop();
    assert_eq!(a, expected);
}

#[test]
fn helper_formatted_retain_chop_test() {
    let mut bold = Attributes::default();
    bold.insert("bold".to_string(), true);

    let mut a = Delta::default();
    a.insert("Test");
    a.retain_attr(4, bold.clone());

    let mut expected = Delta::default();
    expected.insert("Test");
    expected.retain_attr(4, bold.clone());

    a.chop();
    assert_eq!(a, expected);
}
