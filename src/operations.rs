// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[cfg(test)]
use crate::attributes::display_fmt;
use crate::attributes::Attributes;
use crate::error::Error;
use crate::types::attr_map::AttrMap;
use crate::types::attr_val::AttrVal;
use crate::types::ops_kind::OpKind;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut};
#[cfg(test)]
use std::fmt::{Display, Formatter};


/// Operations may have the same structure as an attribute value
/// As a result the `OpsMap` is identical to the `AttrMap` too,
pub type OpsVal = AttrVal;
pub type OpsMap = AttrMap;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum OpType {
    Delete,
    Retain,
    Insert,
}

/// # DeltaOperation()
///
/// Operation definition for the delta format in Rust.
///
/// Insert text:
/// ```json
/// Insert a bolded "Text"
/// { insert: "Text", attributes: { bold: true } }
/// ```
///  Insert a link
/// ```json
/// { insert: "Google", attributes: { link: 'https:///www.google.com' } }
///```
/// Insert an embedded object
/// ```json
/// {
///   insert: { image: 'https:///octodex.github.com/images/labtocat.png' },
///   attributes: { alt: "Lab Octocat" }
/// }
///```
///
/// Insert another embedded object
/// ```json
/// {
///   insert: { video: 'https:///www.youtube.com/watch?v=dMH0bHeiRNg' },
///   attributes: {
///     width: 420,
///     height: 315
///   }
/// }
/// ```
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DeltaOperation {
    #[serde(flatten)]
    pub(crate) kind: OpKind,
    #[serde(default, skip_serializing_if = "Attributes::is_empty")]
    pub(crate) attributes: Attributes,
}

impl DeltaOperation {
    pub fn insert<V: Into<OpsVal>>(value: V) -> Self {
        DeltaOperation {
            kind: OpKind::Insert(value.into()),
            attributes: Attributes::default(),
        }
    }

    pub fn insert_attr<V: Into<OpsVal>>(value: V, attr: Attributes) -> Self {
        DeltaOperation {
            kind: OpKind::Insert(value.into()),
            attributes: attr,
        }
    }

    pub fn retain(value: usize) -> Self {
        DeltaOperation {
            kind: OpKind::Retain(value),
            attributes: Attributes::default(),
        }
    }

    pub fn retain_attr(value: usize, attr: Attributes) -> Self {
        DeltaOperation {
            kind: OpKind::Retain(value),
            attributes: attr,
        }
    }

    /// Delete a value from the input
    pub fn delete(value: usize) -> Self {
        DeltaOperation {
            kind: OpKind::Delete(value),
            attributes: Attributes::default(),
        }
    }

    /// # add_attr()
    /// set the attribute in a shorthand way
    /// ```rust
    /// use crate::delta::operations::DeltaOperation;
    /// let mut op = DeltaOperation::insert("Hallo");
    ///  op.add_attr("font", "green");
    ///  op.add_attr("size", 10);
    /// ```
    pub fn add_attr<K: Into<String>, V: Into<AttrVal>>(&mut self, key: K, value: V) {
        self.attributes.insert(key.into(), value.into());
    }

    /// # set_attributes()
    /// set multiple attributes at once
    pub fn set_attributes<V: Into<Attributes>>(&mut self, values: V) {
        self.attributes = values.into();
    }

    /// # op_len()
    ///
    /// An object is an image or other thing, we treat it as having length 1
    /// In those cases tine insert value is NOT a string.
    pub fn op_len(&self) -> usize {
        match self.kind {
            OpKind::Delete(len) | OpKind::Retain(len) => len,
            OpKind::Insert(OpsVal::String(ref val)) => val.len(),
            OpKind::Insert(_) => 1,
        }
    }

    /// # op_type()
    ///
    /// set the attribute in a shorthand way
    pub fn op_type(&self) -> OpType {
        match self.kind {
            OpKind::Insert(_) => OpType::Insert,
            OpKind::Delete(_) => OpType::Delete,
            OpKind::Retain(_) => OpType::Retain,
        }
    }

    /// # insert_value()
    ///
    /// returns the serde value of the insert operation
    ///
    /// # Panics
    pub fn insert_value(&self) -> &OpsVal {
        if let OpKind::Insert(val) = &self.kind {
            return val;
        }
        panic!("Hey no value found in this operation");
    }

    /// # set_op_kind()
    ///
    /// Sets the operation kind for this delta operation.
    pub fn set_op_kind(&mut self, s: OpKind) {
        *self.kind.borrow_mut() = s;
    }

    /// remove_attribute()
    ///
    /// Removes the attribute that is associated with the given key value.
    pub fn remove_attribute(&mut self, key: &str) {
        self.attributes.remove(key);
    }

    /// # string_val()
    ///
    /// a shorthand way to get the string value out of the serde value
    pub(crate) fn string_val(&self) -> Result<&str, Error> {
        if let OpKind::Insert(OpsVal::String(val)) = &self.kind {
            return Ok(val);
        }
        Err(Error::GetValueWrongType {
            tpe: "string".to_string(),
        })
    }

    pub(crate) fn is_string(&self) -> bool {
        return self.insert_value().is_string();
    }

    pub(crate) fn is_object(&self) -> bool {
        return !self.insert_value().is_string();
    }

    pub fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }

    pub fn get_op_kind(&self) -> &OpKind {
        &self.kind
    }

    fn is_same_attributes(&self, other: &DeltaOperation) -> bool {
        self.attributes.is_equal(&other.attributes)
    }

    pub fn is_same_operation(&self, other: &DeltaOperation) -> bool {
        match &self.kind {
            OpKind::Retain(val) => {
                if let OpKind::Retain(other) = other.kind.borrow() {
                    return val == other;
                }
            }
            OpKind::Insert(val) => {
                if let OpKind::Insert(other) = other.kind.borrow() {
                    return val == other;
                }
            }
            OpKind::Delete(val) => {
                if let OpKind::Delete(other) = other.kind.borrow() {
                    return val == other;
                }
            }
        }
        false
    }

    /// # is_equal()
    ///
    /// Two delta operations are considered equal if both the operation, and the attributes have the same values.
    /// That means that:
    /// ```
    /// use delta::operations::DeltaOperation;
    /// let a = DeltaOperation::insert("hello");
    /// let b = a.clone();
    /// assert!(a.is_equal(&b)); // should pass
    /// ```
    pub fn is_equal(&self, other: &DeltaOperation) -> bool {
        self.is_same_operation(other) && self.is_same_attributes(other)
    }

    /// # is_empty()
    /// Returns true when the operation has zero length
    pub fn is_empty(&self) -> bool {
        self.op_len() == 0
    }
}

//Note display is one form is serialization, but we can not read it back.
//Use json serializer instead
#[cfg(test)]
impl Display for DeltaOperation {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.kind {
            OpKind::Delete(num) => {
                write!(f, "Operation -> Delete[{num}]")
            }
            OpKind::Retain(num) => {
                if self.attributes.is_empty() {
                    write!(f, "Operation -> Retain[{num}]")
                } else {
                    write!(
                        f,
                        "Operation -> Retain[{}], {}",
                        num,
                        display_fmt(&self.attributes)
                    )
                }
            }
            OpKind::Insert(val) => {
                if self.attributes.is_empty() {
                    write!(f, r##"Operation -> Insert[{val}]"##)
                } else {
                    write!(
                        f,
                        r##"Operation -> Insert[{}], {}"##,
                        val,
                        display_fmt(&self.attributes)
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::operations::{DeltaOperation, OpType, OpsMap, OpsVal};
    use crate::types::attr_val::AttrVal;
    use crate::types::ops_kind::OpKind;

    fn insert<V: Into<OpsVal>>(value: V) -> DeltaOperation {
        DeltaOperation::insert(value)
    }

    fn retain(value: usize) -> DeltaOperation {
        DeltaOperation::retain(value)
    }

    fn delete(value: usize) -> DeltaOperation {
        DeltaOperation::delete(value)
    }

    #[test]
    fn op_type_passes() {
        let mut op1 = insert("Hallo");
        op1.add_attr("font", "green");
        assert_eq!(op1.op_type(), OpType::Insert);

        let mut op2: DeltaOperation = insert(5);
        op2.add_attr("font", "green");
        assert_eq!(op2.op_type(), OpType::Insert);

        let mut o = OpsMap::default();
        o.insert("hello".to_string(), "world");
        let mut op3: DeltaOperation = insert(o);
        op3.add_attr("font", "green");
        assert_eq!(op3.op_type(), OpType::Insert);

        let mut op4 = delete(5);
        op4.add_attr("font", "green");
        assert_eq!(op4.op_type(), OpType::Delete);

        let mut op5 = retain(5);
        op5.add_attr("font", "green");
        assert_eq!(op5.op_type(), OpType::Retain);
    }

    #[test]
    fn op_len_passes() {
        let mut op = DeltaOperation::insert("Hallo");
        op.add_attr("fooooont", "greeeeen");
        assert_eq!(op.op_len(), 5);

        let mut op = DeltaOperation::delete(5);
        op.add_attr("fooooont", "greeeeen");
        assert_eq!(op.op_len(), 5);

        let mut op = DeltaOperation::retain(3);
        op.add_attr("fooooont", "greeeeen");
        assert_eq!(op.op_len(), 3);
    }

    #[test]
    fn attr_add_passes() {
        let mut op1 = insert("Hallo");
        op1.add_attr("font", "green");
        let attr = op1.attributes.clone();
        assert_eq!(op1.attributes, attr);

        let mut op2 = insert("World");
        op2.set_attributes(attr.clone());
        assert_eq!(op2.attributes, attr);
    }

    #[test]
    fn insert_val_passes() {
        let mut op = insert("Hallo");
        op.add_attr("font", "green");
        assert_eq!(op.string_val().unwrap(), "Hallo");
    }

    #[test]
    fn change_attributes_passes() {
        let mut op = insert("Hallo");
        op.add_attr("font", "green");

        op.remove_attribute("font");
        assert_eq!(op.attributes.len(), 0);

        let mut op1 = insert("Hallo");
        op1.add_attr("font", "green");
        op1.add_attr("some", "thing");

        op1.remove_attribute("font");
        assert_eq!(op1.attributes.len(), 1);
        assert_eq!(op1.attributes.get("some").unwrap(), &AttrVal::from("thing"));
    }

    #[test]
    fn change_operation_passes() {
        let mut op = insert("Hallo");
        op.add_attr("font", "green");

        op.set_op_kind(OpKind::from("sweet world"));
        assert_eq!(op.attributes.len(), 1);
        assert_eq!(op.string_val().unwrap(), "sweet world");

        let mut op1 = retain(3);
        op1.add_attr("font", "green");
        assert_eq!(op1.op_len(), 3);

        op1.set_op_kind(OpKind::Retain(5));
        assert_eq!(op1.attributes.len(), 1);
        assert_eq!(op1.op_len(), 5);
    }
}
