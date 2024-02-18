#[cfg(test)]
mod tests {
    use anyhow::Result;
    use delta::attributes::Attributes;
    use delta::delta::Delta;
    use delta::document::Document;
    use delta::operations::OpsMap;
    use delta::optransform::OpTransform;
    use delta::types::attr_map::AttrMap;
    use delta::types::attr_val::AttrVal;

    #[test]
    fn insert_passes() {
        let mut a = Delta::default();
        a.insert("A");

        let mut b = Delta::default();
        b.insert("AB");

        let mut expected = Delta::default();
        expected.retain(1);
        expected.insert("B");

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn delete_passes() {
        let mut a = Delta::default();
        a.insert("A");

        let mut b = Delta::default();
        b.insert("AB");

        let mut expected = Delta::default();
        expected.retain(1);
        expected.delete(1);

        let r = match b.diff(&a, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn retain_passes() {
        let mut a = Delta::default();
        a.insert("A");

        let mut b = Delta::default();
        b.insert("A");

        let expected = Delta::default();

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn format_passes() {
        let mut a = Delta::default();
        a.insert("A");

        let mut attr = Attributes::default();
        attr.insert("bold", true);
        let mut b = Delta::default();
        b.insert_attr("A", attr);

        let mut attr = Attributes::default();
        attr.insert("bold", true);
        let mut expected = Delta::default();
        expected.retain_attr(1, attr);

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn object_attributes_passes() {
        let _json_data = r#"{ "font": { "family": "Helvetica", "size": "15px" }}"#;

        let mut m = AttrMap::default();
        m.insert("family", "Helvetica");
        m.insert("size", "15px");
        let mut atr = Attributes::default();
        atr.insert("family", m);

        let mut a = Delta::default();
        a.insert_attr("A", atr);

        let mut m = AttrMap::default();
        m.insert("family", "Helvetica");
        m.insert("size", "15px");
        let mut atr = Attributes::default();
        atr.insert("family", m);

        let mut b = Delta::default();
        b.insert_attr("A", atr);

        let expected = Delta::default();

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn embed_integer_match_passes() {
        let mut a = Delta::default();
        a.insert(1);

        let mut b = Delta::default();
        b.insert(1);

        let expected = Delta::default();

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn embed_integer_mismatch_passes() {
        let mut a = Delta::default();
        a.insert(1);

        let mut b = Delta::default();
        b.insert(2);

        let mut expected = Delta::default();
        expected.delete(1);
        expected.insert(2);

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn embed_object_match_passes() {
        let mut img = OpsMap::default();
        img.insert("image", "http://quilljs.com/");
        let mut a = Delta::default();
        a.insert(img);

        let mut img = OpsMap::default();
        img.insert("image", "http://quilljs.com/");
        let mut b = Delta::default();
        b.insert(img);

        let expected = Delta::default();

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn embed_object_mismatch_passes() {
        let mut img = OpsMap::default();
        img.insert("image", "http://quilljs.com/");
        img.insert("alt", "Overwrite");
        let mut a = Delta::default();
        a.insert(img);

        let mut img = OpsMap::default();
        img.insert("image", "http://quilljs.com/");
        let mut b = Delta::default();
        b.insert(img);

        let mut img = OpsMap::default();
        img.insert("image", "http://quilljs.com/");
        let mut expected = Delta::default();
        expected.insert(img);
        expected.delete(1);

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn embed_object_change_passes() {
        let mut img = OpsMap::default();
        img.insert("image", "http://quilljs.com/");
        let mut a = Delta::default();
        a.insert(img);

        let mut img = OpsMap::default();
        img.insert("image", "http://github.com/");
        let mut b = Delta::default();
        b.insert(img);

        let mut img = OpsMap::default();
        img.insert("image", "http://github.com/");
        let mut expected = Delta::default();
        expected.insert(img);
        expected.delete(1);

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn embed_error_non_document_passes() {
        let mut a = Delta::default();
        a.insert("A");

        let mut b = Delta::default();
        b.retain(1);
        b.insert("B");

        match a.diff(&b, 0) {
            Err(f) => f,
            Ok(_) => panic!("invalid result from diff()"),
        };
    }

    #[test]
    fn embed_inconvenient_indexes_passes() {
        let mut attr1 = Attributes::default();
        attr1.insert("bold", true);

        let mut attr2 = Attributes::default();
        attr2.insert("italic", true);

        let mut a = Delta::default();
        a.insert_attr("12", attr1);
        a.insert_attr("34", attr2);

        let mut attr3 = Attributes::default();
        attr3.insert("color", "red");
        let mut b = Delta::default();
        b.insert_attr("123", attr3);

        let mut attr4 = Attributes::default();
        attr4.insert("bold", AttrVal::Null);
        attr4.insert("color", "red");

        let mut attr5 = Attributes::default();
        attr5.insert("italic", AttrVal::Null);
        attr5.insert("color", "red");

        let mut expected = Delta::default();
        expected.retain_attr(2, attr4);
        expected.retain_attr(1, attr5);
        expected.delete(1);

        let r = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn embed_combination_passes() -> Result<()> {
        let mut red = Attributes::default();
        red.insert("color", "red");
        let mut blue = Attributes::default();
        blue.insert("color", "blue");

        let mut a = Delta::default();
        a.insert_attr("Bad", red.clone());
        a.insert_attr("cat", blue.clone());

        let mut bold = Attributes::default();
        bold.insert("bold", true);
        let mut italic = Attributes::default();
        italic.insert("italic", true);

        let mut b = Delta::default();
        b.insert_attr("Good", bold.clone());
        b.insert_attr("dog", italic.clone());

        // //JSON diff engine is different from the one used here, but applying the delta gives same result
        // let json_data = r#"{  "italic": true, "color": null}"#;
        // let attr5 : Attributes  = serde_json::from_str(&json_data).unwrap();
        // let expected = Delta::default();
        //     .push( DeltaOperation::insert("Good").attrs(bold))
        //     .push( DeltaOperation::delete(2))
        //     .push( DeltaOperation::retain(1).attrs(attr5))
        //     .push( DeltaOperation::delete(3))
        //     .push( DeltaOperation::insert("og").attrs(italic))
        //

        let mut attr5 = Attributes::default();
        attr5.insert("bold", true);
        attr5.insert("color", AttrVal::Null);
        let mut expected = Delta::default();
        expected.insert_attr("Goo", bold);
        expected.delete(2);
        expected.retain_attr(1, attr5);
        expected.delete(3);
        expected.insert_attr("dog", italic);

        let diff = match a.diff(&b, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(&diff, &expected);

        let r = a.compose(&diff)?;
        //dbg!(r);
        assert_eq!(r, b);
        Ok(())
    }
    /*
    var Delta = require('../../dist/Delta');

      it('combination', function() {
        var a = new Delta()
          .insert('Bad', { color: 'red' })
          .insert('cat', { color: 'blue' });
        var b = new Delta()
          .insert('Good', { bold: true })
          .insert('dog', { italic: true });
        var expected = new Delta()
          .insert('Good', { bold: true })
          .delete(2)
          .retain(1, { italic: true, color: null })
          .delete(3)
          .insert('og', { italic: true });
        expect(a.diff(b)).toEqual(expected);
      });

    */

    #[test]
    fn embed_some_document_passes() {
        let mut attr = Attributes::default();
        attr.insert("bold", true);

        let mut a = Delta::default();
        a.insert("A");
        a.insert_attr("B", attr);

        let r = match a.diff(&a, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(r, Delta::default());
    }

    ///
    /// Test in part driven by the fact that in javascript:
    /// a = { bold : true }
    /// b = { bold : true}
    /// a == b --> false --> you need loadash or similar to get the expected behaviour
    #[test]
    fn embed_immutability_passes() {
        let mut attr1 = Attributes::default();
        attr1.insert("color", "red");
        let mut attr2 = Attributes::default();
        attr2.insert("color", "red");

        let mut a1 = Delta::default();
        a1.insert_attr("A", attr1.clone());

        let mut a2 = Delta::default();
        a2.insert_attr("A", attr2.clone());

        let mut attr11 = Attributes::default();
        attr11.insert("bold", true);
        let mut attr22 = Attributes::default();
        attr22.insert("bold", true);

        let mut b1 = Delta::default();
        b1.insert_attr("A", attr11);
        b1.insert("B");

        let mut b2 = Delta::default();
        b2.insert_attr("A", attr22);
        b2.insert("B");

        let mut attr3 = Attributes::default();
        attr3.insert("bold", true);
        attr3.insert("color", AttrVal::Null);
        let mut expected = Delta::default();
        expected.retain_attr(1, attr3);
        expected.insert("B");

        let r = match a1.diff(&b1, 0) {
            Err(_) => panic!("invalid result from diff()"),
            Ok(f) => f,
        };
        assert_eq!(a1, a2);
        assert_eq!(b1, b2);
        assert_eq!(&r, &expected);
        assert_eq!(attr1, attr2);
    }

    #[test]
    fn embed_non_document_passes() {
        let mut a = Delta::default();
        a.insert("Test");

        let mut b = Delta::default();
        b.delete(4);

        match a.diff(&b, 0) {
            Err(f) => f,
            Ok(_) => panic!("invalid result from diff()"),
        };
    }
}
