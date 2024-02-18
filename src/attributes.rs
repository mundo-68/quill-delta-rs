// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(test)]
use std::fmt;
use std::iter::Iterator;
use std::ops::{Deref, DerefMut};

use crate::types::attr_val::AttrVal;
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Attributes {
    #[serde(flatten)]
    attr: HashMap<String, AttrVal>,
}

impl Attributes {
    pub fn is_equal(&self, other: &Attributes) -> bool {
        diff(other, self).is_empty()
    }

    pub fn insert<K: Into<String>, V: Into<AttrVal>>(&mut self, key: K, value: V) {
        self.attr.insert(key.into(), value.into());
    }

    /// Normally derive trait would capture this;
    /// We need to be explicit here to allow this to be used in `operations`
    pub fn is_empty(&self) -> bool {
        self.attr.is_empty()
    }
}

impl Deref for Attributes {
    type Target = HashMap<String, AttrVal>;

    fn deref(&self) -> &Self::Target {
        &self.attr
    }
}

impl DerefMut for Attributes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.attr
    }
}

/// # Panics
/// compose()
///
/// Returns a Delta that is equivalent to applying the operations of
/// own Delta, followed by another Delta.
/// 1) If null values should be removed, remove them from the base
/// 2) if the base does NOT contain the key from the delta then we add it to base
///    regardless if the delta value is "null" or a string, or a JSON object
///
/// base: base delta
/// attrib: delta to apply
///
pub fn compose(attrib: &Attributes, base: &Attributes, keep_null: bool) -> Attributes {
    let mut ret = base.clone();
    if !keep_null {
        //remove all keys in base that point to null ...
        base.keys()
            .filter(|&ak| matches!(base.get(ak), Some(AttrVal::Null)))
            .for_each(|key: &String| {
                ret.remove(key);
            });
    }

    for (key, val) in &**attrib {
        //Note we also skip if attribute is pointing to "None"
        if attrib.get(key).is_some() && base.get(key).is_none() {
            ret.insert(key, val.clone());
        }
    }
    ret
}

/// # Panics
///
/// transform()
///
/// Transform given Delta against own operations.
///
/// Methods
/// transform(other, priority = false)
/// transform(index, priority = false) - Alias for transformPosition
///
/// Parameters
/// base
/// other - Delta to transform
/// priority - Boolean used to break ties.
///         If true, then this takes priority over other, that is, its actions
///         are considered to happen "first."
/// Returns
///    Delta - transformed Delta
///
pub fn transform(a: &Attributes, b: &Attributes, priority: bool) -> Attributes {
    if a.is_empty() {
        return b.clone();
    };
    if b.is_empty() {
        return Attributes::default();
    };

    if !priority {
        // b simply overwrites us without priority
        return b.clone();
    }

    //Fixme: saves a potential panic by not using .unwrap()
    //Fixme: But which implementation is faster ...
    let mut ret = Attributes::default();
    for (key, val) in &**b {
        if a.get(key).is_none() {
            ret.insert(key, val.clone());
        }
    }
    // b.keys().filter(|&k| a.get(k).is_none()).for_each(|k| {
    //     ret.insert(k.clone(), b.get(k).unwrap().clone());
    // });

    ret
}

/// diff()
///
/// Returns a Delta representing the difference between two documents.
///
/// Parameters
///   other - Document Delta to diff against
///   index - Suggested index where change took place
/// Returns Delta - difference between the two documents
///   base: first quill delta
///   attrib: second quill delta
///
pub fn diff(attrib: &Attributes, base: &Attributes) -> Attributes {
    let mut ret = Attributes::default();
    attrib.keys().chain(base.keys()).for_each(|key| {
        if attrib.get(key) != base.get(key) {
            match base.get(key) {
                None => {
                    ret.insert(key.clone(), AttrVal::Null);
                }
                Some(x) => {
                    ret.insert(key.clone(), x.clone());
                }
            }
        }
    });
    ret
}

/// # Panics
/// invert()
///
/// Returned an inverted quill delta that has the opposite effect of against
/// a base document quill delta.
/// That is `base.compose(quill_delta-rs).compose(inverted) === base`.
///
///invert(r#"{"bold": null}"#, r#"{"bold": true}"#), r#"{"bold": true}"#)
pub fn invert(attr: &Attributes, base: &Attributes) -> Attributes {
    let mut base_inverted = Attributes::default();
    //Fixme: saves a potential panic by not using .unwrap()
    //Fixme: But which implementation is faster ...
    for (key, val) in &**base {
        if base.get(key) != attr.get(key) && attr.get(key).is_some() {
            base_inverted.insert(key, val.clone());
        }
    }
    // base.keys().for_each(|key| {
    //     if base.get(key) != attr.get(key) && attr.get(key).is_some() {
    //         base_inverted.insert(key.clone(), base.get(key).unwrap().clone());
    //     }
    // });

    attr.keys().for_each(|key| {
        if attr.get(key) != base.get(key) && base.get(key).is_none() {
            base_inverted.insert(key.clone(), AttrVal::Null);
        }
    });
    base_inverted
}

impl From<HashMap<String, AttrVal>> for Attributes {
    fn from(m: HashMap<String, AttrVal>) -> Self {
        Attributes { attr: m }
    }
}

#[cfg(test)]
impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", display_fmt(self))
    }
}

//It is not possible to extend a trait defined in another crate
//In this case that is HashMap, so we define a function instead
#[cfg(test)]
pub(crate) fn display_fmt(attr: &Attributes) -> String {
    let mut at = String::new();
    for (k, v) in attr.iter() {
        if at.is_empty() {
            at = format!(r#"{k:?}:{v}"#);
        } else {
            at = format!(r#"{at}; {k:?}:{v}"#);
        }
    }
    format!(r#" Attr[{at}] "#)
}

#[cfg(test)]
mod tests {
    use crate::attributes::{compose, diff, invert, transform, Attributes};
    use crate::types::attr_val::AttrVal;

    #[test]
    fn compose_left_undefined_passes() {
        let mut att = Attributes::default();
        att.insert("bold", true);
        att.insert("color", "red");

        let res = att.clone();
        compose(&Attributes::default(), &att, true);
        assert_eq!(att, res);
    }

    #[test]
    fn compose_right_undefined_passes() {
        let mut att = Attributes::default();
        att.insert("bold", true);
        att.insert("color", "red");

        let res = att.clone();
        compose(&att, &Attributes::default(), true);
        assert_eq!(att, res);
    }

    #[test]
    fn compose_both_undefined_passes() {
        let attributes = Attributes::default();
        compose(&Attributes::default(), &attributes, true);
        assert_eq!(attributes, Attributes::default());
    }

    #[test]
    fn compose_missing_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut italics = Attributes::default();
        italics.insert("italic", true);

        let mut combi = Attributes::default();
        combi.insert("bold", true);
        combi.insert("color", "red");
        combi.insert("italic", true);

        let res = compose(&italics, &attributes, true);
        assert_eq!(res, combi);
    }

    #[test]
    fn compose_overwrite_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut italics = Attributes::default();
        italics.insert("bold", false);
        italics.insert("color", "blue".to_string());

        let mut combi = Attributes::default();
        combi.insert("bold", false);
        combi.insert("color", "blue");

        compose(&attributes, &italics, false);
        assert_eq!(italics, combi);
    }

    #[test]
    fn compose_remove_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut italics = Attributes::default();
        italics.insert("bold", AttrVal::Null);

        let mut combi = Attributes::default();
        combi.insert("color", "red");

        let res = compose(&attributes, &italics, false);
        assert_eq!(res, combi);
    }

    #[test]
    fn compose_remove_to_none_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut italics = Attributes::default();
        italics.insert("bold", AttrVal::Null);
        italics.insert("color", AttrVal::Null);

        let combi: Attributes = Attributes::default();

        let res = compose(&attributes, &italics, false);

        assert_eq!(res, combi);
    }

    #[test]
    fn compose_remove_missing_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut italics = Attributes::default();
        italics.insert("italic", AttrVal::Null);

        let res = compose(&attributes, &italics, false);
        assert_eq!(res, attributes);
    }

    #[test]
    fn diff_left_undefined_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        assert_eq!(diff(&Attributes::default(), &attributes), attributes);
    }

    #[test]
    fn diff_right_undefined_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut expected = Attributes::default();
        expected.insert("bold", AttrVal::Null);
        expected.insert("color", AttrVal::Null);

        assert_eq!(diff(&attributes, &Attributes::default()), expected);
    }

    #[test]
    fn diff_same_format_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        assert_eq!(diff(&attributes, &attributes), Attributes::default());
    }

    #[test]
    fn diff_add_format_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut added = Attributes::default();
        added.insert("bold", true);
        added.insert("italic", true);
        added.insert("color", "red");

        let mut expected = Attributes::default();
        expected.insert("italic", true);

        assert_eq!(diff(&attributes, &added), expected);
    }

    #[test]
    fn diff_remove_format_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut removed = Attributes::default();
        removed.insert("bold", true);

        let mut expected = Attributes::default();
        expected.insert("color", AttrVal::Null);

        assert_eq!(diff(&attributes, &removed), expected);
    }

    #[test]
    fn diff_overwrite_format_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("color", "red");

        let mut removed = Attributes::default();
        removed.insert("bold", true);
        removed.insert("color", "blue");

        let mut expected = Attributes::default();
        expected.insert("color", "blue");

        assert_eq!(diff(&attributes, &removed), expected);
    }

    #[test]
    fn invert_passes() {
        let mut base = Attributes::default();
        base.insert("bold", true);

        assert_eq!(invert(&Attributes::default(), &base), Attributes::default());
    }

    #[test]
    fn invert_base_undefined_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);

        let mut expected = Attributes::default();
        expected.insert("bold", AttrVal::Null);

        assert_eq!(invert(&attributes, &Attributes::default()), expected);
    }

    #[test]
    fn invert_both_undefined_passes() {
        assert_eq!(
            invert(&Attributes::default(), &Attributes::default()),
            Attributes::default()
        );
    }

    #[test]
    fn invert_merge_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);

        let mut base = Attributes::default();
        base.insert("italic", true);

        let mut expected = Attributes::default();
        expected.insert("bold", AttrVal::Null);

        assert_eq!(invert(&attributes, &base), expected);
    }

    #[test]
    fn invert_null_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", AttrVal::Null);

        let mut base = Attributes::default();
        base.insert("bold", true);

        let mut expected = Attributes::default();
        expected.insert("bold", true);

        assert_eq!(invert(&attributes, &base), expected);
    }

    #[test]
    fn invert_replace_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("color", "red");

        let mut base = Attributes::default();
        base.insert("color", "blue");

        let mut expected = Attributes::default();
        expected.insert("color", "blue");

        assert_eq!(invert(&attributes, &base), expected);
    }

    #[test]
    fn invert_combined_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("bold", true);
        attributes.insert("italic", AttrVal::Null);
        attributes.insert("color", "red");
        attributes.insert("size", "12px");

        let mut base = Attributes::default();
        base.insert("font", "serif");
        base.insert("italic", true);
        base.insert("color", "blue");
        base.insert("size", "12px");

        let mut expected = Attributes::default();
        expected.insert("bold", AttrVal::Null);
        expected.insert("italic", true);
        expected.insert("color", "blue");

        assert_eq!(invert(&attributes, &base), expected);
    }

    #[test]
    fn invert_noop_passes() {
        let mut attributes = Attributes::default();
        attributes.insert("color", "red");

        let mut base = Attributes::default();
        base.insert("color", "red");

        assert_eq!(invert(&attributes, &base), Attributes::default());
    }

    #[test]
    fn transform_left_undefined_passes() {
        let mut left = Attributes::default();
        left.insert("bold", true);
        left.insert("color", "red");
        left.insert("font", AttrVal::Null);

        let attributes = Attributes::default();

        let res = transform(&attributes, &left, false);

        assert_eq!(res, left);
    }

    #[test]
    fn transform_right_undefined_passes() {
        let mut right = Attributes::default();
        right.insert("bold", true);
        right.insert("color", "red");
        right.insert("font", AttrVal::Null);

        let res = transform(&right, &Attributes::default(), false);

        assert_eq!(res, Attributes::default());
    }

    #[test]
    fn transform_both_undefined_passes() {
        let attributes = Attributes::default();
        transform(&attributes, &Attributes::default(), false);
        assert_eq!(attributes, Attributes::default());
    }

    #[test]
    fn transform_with_priority_passes() {
        let mut left = Attributes::default();
        left.insert("bold", true);
        left.insert("color", "red");
        left.insert("font", AttrVal::Null);

        let mut right = Attributes::default();
        right.insert("color", "blue");
        right.insert("font", "serif");
        right.insert("italic", true);

        let mut expected = Attributes::default();
        expected.insert("italic", true);

        let res = transform(&left, &right, true);
        assert_eq!(res, expected);
    }

    #[test]
    fn transform_without_priority_passes() {
        let mut left = Attributes::default();
        left.insert("bold", true);
        left.insert("color", "red");
        left.insert("font", AttrVal::Null);

        let mut right = Attributes::default();
        right.insert("color", "blue");
        right.insert("font", "serif");
        right.insert("italic", true);

        let mut expected = Attributes::default();
        expected.insert("italic", true);

        let res = transform(&left, &right, false);

        assert_eq!(res, right);
    }
}
