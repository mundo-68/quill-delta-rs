// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::delta::Delta;
use crate::iterator::DeltaIterator;
use crate::operations::DeltaOperation;

pub trait DeltaTransformations {
    fn filter<F>(&self, predicate: F) -> Delta
    where
        F: Fn(&DeltaOperation, usize) -> bool;

    //
    // Execute for each DeltaOperation in a delta using a
    // closure f(&mut DeltaOperation)
    fn for_each<F>(&self, predicate: F)
    where
        F: Fn(&DeltaOperation);

    //
    // length of the delta operations regardless of the OpKind
    fn delta_length(&self) -> usize;

    //
    // Execute map for DeltaOperation in a delta using a
    // closure f(&DeltaOperation, usize)->bool
    // where index:usize is the position of the DeltaOperation in the Delta document
    fn map<T, F>(&self, predicate: F) -> Vec<T>
    where
        F: Fn(&DeltaOperation, usize) -> T;

    fn partition<F>(&self, predicate: F) -> (Delta, Delta)
    where
        F: Fn(&DeltaOperation) -> bool;

    //
    // Execute map for DeltaOperation in a delta using a
    // closure f(T, &DeltaOperation, usize)->bool
    // where
    //     T is the accumulator
    //     DeltaOperation the current DeltaOperation
    //     index:usize is the position of the DeltaOperation in the Delta document
    fn reduce<'a, T, F>(&self, predicate: F, init_val: &'a mut T) -> &'a mut T
    where
        F: Fn(&mut T, &DeltaOperation, usize) -> T;

    //
    // Returns copy of quill delta with subset of operations.
    // use `end = usize::MAX` when the slice goes all the way up to the end
    //
    // `start` - Start index of subset, default to 0
    // `end` - End index of subset, defaults to rest of operations; use `usize::MAX` for all
    fn slice(&self, start: usize, end: usize) -> Delta;
}

impl DeltaTransformations for Delta {
    //
    // Filter a delta using a closure:
    //     f(&DeltaOperation,usize)->bool
    // where
    //     DeltaOperation current value to be filtered
    //     index:usize is the position of the DeltaOperation in the Delta document
    // Only when the filter returns true, will the output delta be collected in the end result
    fn filter<F>(&self, predicate: F) -> Delta
    where
        F: Fn(&DeltaOperation, usize) -> bool,
    {
        let mut i: usize = 0;
        self.iter()
            .map(|d| {
                i += 1;
                (d, i)
            })
            .filter(|d| predicate(d.0, d.1))
            .map(|d| d.0.clone())
            .collect()
    }

    fn for_each<F>(&self, predicate: F)
    where
        F: Fn(&DeltaOperation),
    {
        self.iter().for_each(predicate);
    }

    fn delta_length(&self) -> usize {
        let mut len: usize = 0;
        for d in self.iter() {
            len += d.op_len();
        }
        len
    }

    fn map<T, F>(&self, predicate: F) -> Vec<T>
    where
        F: Fn(&DeltaOperation, usize) -> T,
    {
        let mut i: usize = 0;
        self.iter()
            .map(|d| {
                i += 1;
                (d, i)
            })
            .map(|d| predicate(d.0, d.1))
            .collect()
    }

    fn partition<F>(&self, predicate: F) -> (Delta, Delta)
    where
        F: Fn(&DeltaOperation) -> bool,
    {
        let mut passed: Delta = Delta::default();
        let mut failed: Delta = Delta::default();
        self.iter().for_each(|d| {
            if predicate(d) {
                passed.push(d.clone());
            } else {
                failed.push(d.clone());
            }
        });
        (passed, failed)
    }

    fn reduce<'a, T, F>(&self, predicate: F, init_val: &'a mut T) -> &'a mut T
    where
        F: Fn(&mut T, &DeltaOperation, usize) -> T,
    {
        let mut i: usize = 0;
        self.iter().for_each(|d| {
            i += 1;
            predicate(init_val, d, i);
        });
        init_val
    }

    fn slice(&self, start: usize, end: usize) -> Delta {
        //define length of the slice, 0 is up to the end
        let mut einde = end;
        if end == 0 {
            einde = self.delta_length();
        }

        let mut delta = Delta::default();
        let iter = DeltaIterator::new(self);
        let mut index: usize = 0;
        while index < einde && iter.has_next() {
            let next_op: DeltaOperation;
            if index < start {
                next_op = iter.next_len(start - index);
                index += &next_op.op_len();
            } else {
                next_op = iter.next_len(einde - index);
                index += &next_op.op_len();
                delta.push(next_op);
            }
        }
        delta
    }
}
