#[cfg(test)]
mod tests {
    use delta::attributes::Attributes;
    use delta::delta::Delta;
    use delta::operations::{DeltaOperation, OpsMap, OpsVal};
    use delta::utils::DeltaTransformations;

    #[test]
    fn construct_passes() {
        let _ops: &str = r#"{
            "ops" : [
                { "insert": "abc" },
                { "retain": 1, "attributes": { "color": "red" } },
                { "delete": 4 },
                { "insert": "def", "attributes": { "bold": true } },
                { "retain": 6 }
              ]}"#;

        let mut delta = Delta::default();

        let op = DeltaOperation::insert("abc");
        delta.push(op);

        let mut op = DeltaOperation::retain(1);
        op.add_attr("color", "red");
        delta.push(op);

        let op = DeltaOperation::delete(4);
        delta.push(op);

        let mut op = DeltaOperation::insert("def");
        op.add_attr("bold", true);
        delta.push(op);

        let op = DeltaOperation::retain(6);
        delta.push(op);

        let mut d = Delta::default();
        assert_eq!(d.delta_length(), 0);
        assert_eq!(d.len(), 0);

        d.insert("");
        d.delete(0);
        d.retain(0);
    }

    #[test]
    fn insert_passes() {
        let mut d = Delta::default();
        d.insert("Test");
        assert_eq!(d.delta_length(), 4);
        assert_eq!(d.len(), 1);
        assert_eq!(d.first().unwrap().insert_value().str_val().unwrap(), "Test");

        let mut d = Delta::default();
        d.insert("Test");
        d.insert("Test");
        assert_eq!(d.delta_length(), 8);
        assert_eq!(d.len(), 1);
        assert_eq!(
            d.first().unwrap().insert_value().str_val().unwrap(),
            "TestTest"
        );

        //Embedded objects in delta can either be a map, or a integer value (both non "String")
        let mut d = Delta::default();
        d.insert(1);
        assert_eq!(d.delta_length(), 1);
        assert_eq!(d.len(), 1);
        assert_eq!(d.first().unwrap().insert_value(), &OpsVal::from(1));

        let _json_data = r#"{ "src": "http://quilljs.com/image.png" }"#;
        let mut attr = Attributes::default();
        attr.insert("src", "http://quilljs.com/image.png");
        let mut d = Delta::default();
        d.insert_attr(1, attr.clone());
        assert_eq!(d.delta_length(), 1);
        assert_eq!(d.len(), 1);
        assert_eq!(d.first().unwrap().insert_value(), &OpsVal::from(1));
        assert_eq!(d.first().unwrap().get_attributes(), &attr);

        let mut embed = OpsMap::default();
        embed.insert("src", "http://quilljs.com/image.png");
        let mut op = DeltaOperation::insert(embed.clone());
        op.add_attr("alt", "quill");
        let mut d = Delta::default();
        d.push(op);
        //d.insert_attr(embed.clone(), alt.clone());
        assert_eq!(d.delta_length(), 1);
        assert_eq!(d.len(), 1);
        let res = d.first().unwrap().insert_value().map_val().unwrap();
        assert_eq!(res, &embed);
        let mut alt = Attributes::default();
        alt.insert("alt", "quill");
        assert_eq!(d.first().unwrap().get_attributes(), &alt);
    }

    #[test]
    pub fn build_insert_passes() {
        let mut attr = Attributes::default();
        attr.insert("bold", true);

        let mut delta = Delta::default();
        delta.insert_attr("test", attr.clone());

        assert_eq!(
            delta.first().unwrap().insert_value().str_val().unwrap(),
            "test"
        );
        assert_eq!(delta.first().unwrap().get_attributes(), &attr);
    }

    #[test]
    pub fn build_insert_after_delete_passes() {
        let mut delta = Delta::default();
        delta.delete(1);
        delta.insert("a");

        let mut expected = Delta::default();
        expected.insert("a");
        expected.delete(1);

        assert_eq!(delta, expected);
    }

    #[test]
    pub fn build_insert_after_delete_with_merge_passes() {
        let mut delta = Delta::default();
        delta.insert("a");
        delta.delete(1);
        delta.insert("b");

        let mut expected = Delta::default();
        expected.insert("ab");
        expected.delete(1);

        assert_eq!(delta, expected);
    }

    #[test]
    pub fn build_insert_text_after_delete_no_merge_passes() {
        //Normally inserts are merged, but strings and integers dont merge (just strings do)
        let mut delta1 = Delta::default();
        delta1.insert(1);
        delta1.delete(1);
        delta1.insert("a");

        //expected sequence as shown below
        let mut delta2 = Delta::default();
        delta2.insert(1);
        delta2.insert("a");
        delta2.delete(1);

        dbg!(&delta1);

        assert_eq!(delta1, delta2);
    }

    #[test]
    pub fn build_delete_passes() {
        let mut delta = Delta::default();
        delta.delete(0);

        assert_eq!(delta.len(), 0);
        assert_eq!(delta.delta_length(), 0);
    }

    #[test]
    pub fn build_delete_positive_passes() {
        let mut delta = Delta::default();
        delta.delete(1);

        assert_eq!(delta.len(), 1);
        assert_eq!(delta.delta_length(), 1);
    }

    #[test]
    pub fn build_retain_passes() {
        let mut delta = Delta::default();
        delta.retain(0);

        assert_eq!(delta.len(), 0);
        assert_eq!(delta.delta_length(), 0);
    }

    #[test]
    pub fn build_retain_pos_passes() {
        let mut delta = Delta::default();
        delta.retain(2);

        assert_eq!(delta.len(), 1);
        assert_eq!(delta.delta_length(), 2);
    }

    #[test]
    pub fn build_retain_attrib_passes() {
        let mut attr = Attributes::default();
        attr.insert("bold", true);

        let mut delta = Delta::default();
        delta.retain_attr(2, attr.clone());

        assert_eq!(delta.len(), 1);
        assert_eq!(delta.delta_length(), 2);
        assert_eq!(delta.first().unwrap().get_attributes(), &attr);
    }

    #[test]
    pub fn build_retain_length_passes() {
        // Delete prevents chop
        let mut delta = Delta::default();
        delta.retain(2);
        delta.delete(1);

        assert_eq!(delta.len(), 2);
        assert_eq!(delta.delta_length(), 3);
    }

    #[test]
    pub fn build_push_passes() {
        let mut delta = Delta::default();
        delta.insert("test");

        assert_eq!(delta.len(), 1);
        assert_eq!(delta.delta_length(), 4);
        assert_eq!(
            delta.first().unwrap().insert_value().str_val().unwrap(),
            "test"
        );
    }

    #[test]
    pub fn build_push_consequtive_delete_passes() {
        let mut delta = Delta::default();
        delta.delete(2);

        delta.delete(3);
        assert_eq!(delta.len(), 1);
        assert_eq!(delta.delta_length(), 5);
    }

    #[test]
    pub fn build_push_consequtive_text_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.insert_attr("a", bold.clone());
        delta.insert_attr("b", bold.clone());

        assert_eq!(delta.len(), 1);
        assert_eq!(delta.delta_length(), 2);
        assert_eq!(delta.first().unwrap().get_attributes(), &bold);
        assert_eq!(
            delta.first().unwrap().insert_value().str_val().unwrap(),
            "ab"
        );
    }

    #[test]
    pub fn build_push_consecutive_remains_with_matching_attributes_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.retain_attr(1, bold.clone());
        delta.retain_attr(3, bold.clone());

        assert_eq!(delta.len(), 1);
        assert_eq!(delta.delta_length(), 4);
        assert_eq!(delta.first().unwrap().get_attributes(), &bold);
    }

    #[test]
    pub fn build_push_consecutive_texts_mismatch_attributes_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.insert_attr("a", bold.clone());
        delta.insert("b");

        assert_eq!(delta.len(), 2);
        assert_eq!(delta.delta_length(), 2);
        assert_eq!(delta.first().unwrap().get_attributes(), &bold);
        assert_eq!(
            delta.get(1).unwrap().get_attributes(),
            &Attributes::default()
        );
    }

    #[test]
    pub fn build_push_consecutive_retains_mismatched_attributes_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.retain_attr(1, bold.clone());
        delta.retain(3);

        assert_eq!(delta.len(), 2);
        assert_eq!(delta.delta_length(), 4);
        assert_eq!(delta.first().unwrap().get_attributes(), &bold);
        assert_eq!(
            delta.get(1).unwrap().get_attributes(),
            &Attributes::default()
        );
    }
}
