// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::attributes::{diff, invert, Attributes};
use crate::delta::Delta;
use crate::error::Error;
use crate::iterator::DeltaIterator;
use crate::operations::{DeltaOperation, OpType, OpsVal};
use crate::types::ops_kind::OpKind;
use crate::utils::DeltaTransformations;
use anyhow::Result;
use diffs::{myers, Diff, Replace};

/// These methods called on or with non-document Deltas will result in undefined behavior.
pub trait Document {
    /// Returns a Delta representing the concatenation of
    /// this and another document Delta's operations.
    /// ```
    /// extern crate delta;
    /// use delta::delta::Delta;
    /// use delta::document::{Document};
    /// use delta::attributes::{Attributes};
    ///
    /// let mut bold = Attributes::default();
    /// bold.insert("bold".to_string(), true);
    ///
    /// let mut a = Delta::default();
    /// a.insert_attr("Test", bold.clone());
    ///
    /// let mut concat = Delta::default();
    /// concat.insert_attr("!", bold.clone());
    ///
    /// let mut expected = Delta::default();
    /// expected.insert_attr("Test!", bold.clone());
    ///
    /// let c = a.clone().concat(concat).to_owned();
    /// assert_eq!( c, expected);
    /// ```
    fn concat(&mut self, other: Delta) -> &mut Delta;

    /// Returns a Delta representing the difference between two documents.
    /// Optionally, accepts a suggested index where change took place, often
    /// representing a cursor position before change.
    ///
    /// ```
    /// extern crate delta;
    /// use delta::delta::Delta;
    /// use delta::document::{Document};
    ///
    /// let  mut a = Delta::default();
    ///  a.insert("Hallo");
    /// let  mut b = Delta::default();
    ///  b.insert("Hallo!");
    ///
    /// let _diff = a.diff(&b, 0);
    /// // result = { ops: [{ retain: 5 }, { insert: '!' }] }
    /// ```
    ///
    /// # Errors
    /// `ErrorDelta::NotADocument` --> if `other` is not a document (i.e. contains other operations than Insert)
    fn diff(&self, other: &Delta, _cursor: usize) -> Result<Delta, Error>;

    /// # Errors
    ///
    /// run for each line in the text a method --> not that the line length
    /// is defined by a line brake character --> normally '\n'
    /// Lines are processed until the predicate returns false as output.
    ///
    /// This method is expected to run on aa document. So we bail out when
    /// we first encounter a non insert character.
    ///
    /// With closure Fn(&Delta, Attributes, usize) -> bool
    ///     Delta containing the line
    ///     Attribute at the end of line character (might be a separate DeltaOperation)
    ///     integer with the line number
    fn each_line<F>(&self, predicate: F, new_line_char: Option<char>) -> Result<(), Error>
    where
        F: Fn(&Delta, &Attributes, usize) -> bool;

    /// Returns an inverted delta that has the opposite effect of against a base document delta.
    /// That is base.compose(delta).compose(inverted) === base.
    ///
    /// Example
    ///
    /// const base = new Delta().insert('Hello\n')
    ///                         .insert('World');
    /// const delta = new Delta().retain(6, { bold: true }).insert('!').delete(5);
    ///
    /// const inverted = delta.invert(base);  // { ops: [
    ///                                       //   { retain: 6, attributes: { bold: null } },
    ///                                       //   { insert: 'World' },
    ///                                       //   { delete: 1 }
    ///                                       // ]}
    ///                                       // base.compose(delta).compose(inverted) === base
    fn invert(&self, base: &Delta) -> Delta;

    /// Length of content in this delta
    fn document_length(&self) -> usize;
}

impl Document for Delta {
    fn concat(&mut self, mut other: Delta) -> &mut Delta {
        if !other.is_empty() {
            self.push(other.remove(0)); //merges repeated retain, delete, insert
            self.extend(other.to_vec()); //should have no repetitions
        }
        self
    }

    fn diff<'a>(&self, other: &Delta, _cursor: usize) -> Result<Delta, Error> {
        //Collect all inserts in to 1 long string
        let aa = to_diff_string(self)?;
        let bb = to_diff_string(other)?;
        //Split strings in characters to diff over
        let a: Vec<char> = aa.chars().collect();
        let b: Vec<char> = bb.chars().collect();
        //result document
        let mut delta = Delta::default();

        let mut ddd: D = D {
            res: &mut delta,                       //delta to be returned
            other: &mut DeltaIterator::new(other), //iterator other delta from input
            me: &mut DeltaIterator::new(self),     //self delta ...
        };

        let mut diff = Replace::new(&mut ddd);
        myers::diff(&mut diff, &a, 0, a.len(), &b, 0, b.len()).unwrap();

        delta.chop();
        Ok(delta)
    }

    fn each_line<F>(&self, predicate: F, new_line_char: Option<char>) -> Result<(), Error>
    where
        F: Fn(&Delta, &Attributes, usize) -> bool,
    {
        //Standard, or prescribed new line character?
        let mut new_line = '\n';
        if let Option::Some(nl) = new_line_char {
            new_line = nl;
        }

        //collect a line ... repeatedly
        let iter = DeltaIterator::new(self);
        let mut line = Delta::default();
        let mut i = 0;
        while iter.has_next() {
            if iter.peek_type() != OpType::Insert {
                return Ok(());
            }
            let Some(this_op) = iter.peek() else {
                return Err(Error::IteratorIsEmpty);
            };
            let start = this_op.op_len() - iter.peek_len();
            if this_op.is_object() {
                line.push(iter.next_len(0));
            } else {
                //no more options, it must be a string, or and object ...
                let newline_found = this_op.string_val()?[start..].find(new_line);
                match newline_found {
                    None => {
                        line.push(iter.next_len(0));
                    }
                    Some(t) => {
                        let len = t;
                        if len > 0 {
                            line.push(iter.next_len(len));
                        } else {
                            //len=0 --> we are ON the next line marker
                            let go_on = predicate(&line, &iter.next_len(1).attributes, i);
                            if !go_on {
                                return Ok(());
                            }
                            i += 1;
                            line = Delta::default();
                        }
                    }
                }
            }
        }
        //run the predicate on the remaining line (last char need not be a line break)
        if line.delta_length() > 0 {
            predicate(&line, &Attributes::default(), i);
        }
        Ok(())
    }

    fn invert(&self, base: &Delta) -> Delta {
        let mut inverted = Delta::default();

        let predicate = |base_index: usize, op: &DeltaOperation| -> usize {
            if op.op_type() == OpType::Insert {
                inverted.delete(op.op_len());
            } else if op.op_type() == OpType::Retain && op.attributes.is_empty() {
                inverted.retain(op.op_len());
                return base_index + op.op_len();
            } else if op.op_type() == OpType::Delete
                || (op.op_type() == OpType::Retain && !op.attributes.is_empty())
            {
                let length = op.op_len();
                let slice = base.slice(base_index, base_index + length);
                slice.iter().for_each(|base_op| {
                    if op.op_type() == OpType::Delete {
                        inverted.push(base_op.clone());
                    } else if op.op_type() == OpType::Retain && !op.attributes.is_empty() {
                        inverted.retain_attr(
                            base_op.op_len(),
                            invert(&op.attributes, &base_op.attributes),
                        );
                    }
                });
                return base_index + length;
            }
            base_index
        };
        self.iter().fold(0, predicate);
        return inverted.chop().to_owned();
    }

    fn document_length(&self) -> usize {
        let mut len: usize = 0;
        for d in self.iter() {
            match d.op_type() {
                OpType::Insert => len += d.op_len(),
                OpType::Delete => len -= d.op_len(),
                OpType::Retain => {}
            }
        }
        len
    }
}

/// placeholder char to embed in diff()
const NULL_CHARACTER: char = '\0';

struct D<'a> {
    pub res: &'a mut Delta,
    pub other: &'a DeltaIterator<'a>,
    pub me: &'a DeltaIterator<'a>,
}

impl<'a> Diff for D<'a> {
    type Error = ();
    fn equal(&mut self, _o: usize, _new: usize, len: usize) -> Result<(), ()> {
        let mut l = len;
        while l > 0 {
            //dbg!( "diff --> Equal ");
            let v = [self.me.peek_len(), self.other.peek_len(), len];
            let op_len = *v.iter().min().unwrap();
            let this_op = self.me.next_len(op_len);
            let other_op = self.other.next_len(op_len);
            if this_op.op_type() == OpType::Insert
                && other_op.op_type() == OpType::Insert
                && this_op.is_same_operation(&other_op)
            {
                let mut delta = DeltaOperation::retain(op_len);
                delta.set_attributes(diff(&this_op.attributes, &other_op.attributes));
                self.res.push(delta);
            } else {
                // dbg!(&other_op);
                self.res.push(other_op.clone());
                self.res.delete(op_len);
            }
            l -= op_len;
        }
        Ok(())
    }
    fn delete(&mut self, _o: usize, len: usize, _new: usize) -> Result<(), ()> {
        let mut l = len;
        while l > 0 {
            //dbg!( "diff --> Delete ");
            let v = [self.me.peek_len(), len];
            let op_len = *v.iter().min().unwrap();
            self.me.next_len(op_len);
            let op = DeltaOperation::delete(op_len);
            // dbg!(&op);
            self.res.push(op);
            l -= op_len;
        }
        Ok(())
    }
    fn insert(&mut self, _o: usize, _n: usize, len: usize) -> Result<(), ()> {
        let mut l = len;
        while l > 0 {
            //dbg!( "diff --> Insert ");
            // dbg!(_len);
            // dbg!(self.other.peek_len());
            let v = [self.other.peek_len(), len];
            let op_len = *v.iter().min().unwrap();
            // dbg!(op_len);
            let op = self.other.next_len(op_len).clone();
            // dbg!(&op);
            self.res.push(op);
            l -= op_len;
        }
        // dbg!( self.me.debug_index());
        // dbg!( self.other.debug_index());
        Ok(())
    }
}

/// Private method
/// To convert a list of DeltaOperation in to 1 single string
/// Regardless of the attributes in each DeltaOperation
///
/// Generate a string with all insert concatenated
/// and non string things "Insert(Hasmap)"  represented by `NULL_CHARACTER`.
fn to_diff_string(delta: &Delta) -> Result<String, Error> {
    let mut res = String::new();
    for op in delta.iter() {
        match op {
            DeltaOperation {
                kind: OpKind::Insert(OpsVal::String(val)),
                ..
            } => {
                res.push_str(&val[..]);
            }
            DeltaOperation {
                kind: OpKind::Insert(_),
                ..
            } => {
                res.push(NULL_CHARACTER);
            }
            //A document is valid when all delta in the document are "insert" operations
            _ => return Err(Error::NotADocument),
        }
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use crate::delta::Delta;
    use crate::document::{Document, NULL_CHARACTER};
    use crate::error::Error;

    #[test]
    fn embed_false_positive_passes() -> Result<(), Error> {
        let mut a = Delta::default();
        a.insert(1);

        let mut b = Delta::default();
        b.insert(NULL_CHARACTER.to_string());

        //dbg!(&b);

        let mut expected = Delta::default();
        expected.insert(NULL_CHARACTER.to_string());
        expected.delete(1);

        let r = a.diff(&b, 0)?;
        assert_eq!(r, expected);
        Ok(())
    }
}
