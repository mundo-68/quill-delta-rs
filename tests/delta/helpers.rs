#[cfg(test)]
mod tests {
    use delta::attributes::Attributes;
    use delta::delta::Delta;
    use delta::document::Document;
    use delta::operations::{DeltaOperation, OpsMap};
    use delta::types::attr_val::AttrVal;
    use delta::utils::DeltaTransformations;

    #[test]
    fn helper_concat_passes() {
        let mut a = Delta::default();
        a.insert("Test");

        let concat = Delta::default();

        let mut expected = Delta::default();
        expected.insert("Test");

        a.concat(concat);
        assert_eq!(a, expected);
    }

    #[test]
    fn helper_unmergeable_passes() {
        let mut a = Delta::default();
        a.insert("Test");

        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut concat = Delta::default();
        concat.insert_attr("!", bold.clone());

        let mut expected = Delta::default();
        expected.insert("Test");
        expected.insert_attr("!", bold);

        let c = a.clone().concat(concat).to_owned();
        assert_eq!(c, expected);
    }

    #[test]
    fn helper_mergeable_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut a = Delta::default();
        a.insert_attr("Test", bold.clone());

        let mut concat = Delta::default();
        concat.insert_attr("!", bold.clone());

        let mut expected = Delta::default();
        expected.insert_attr("Test!", bold.clone());

        let c = a.clone().concat(concat).to_owned();
        assert_eq!(c, expected);
    }

    #[test]
    fn helper_eachline_passes() -> anyhow::Result<()> {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut align = Attributes::default();
        align.insert("align", "right");

        let mut octo = OpsMap::default();
        octo.insert("image", "octocat.png");

        let mut a = Delta::default();
        a.insert("Hello\n\n");
        a.insert_attr("World", bold.clone());
        a.insert(octo.clone());
        a.insert_attr("\n", align.clone());
        a.insert("!");

        let mut expect1 = Delta::default();
        expect1.insert("Hello");

        let expect2 = Delta::default();
        let mut expect3 = Delta::default();
        expect3.insert_attr("World", bold.clone());
        expect3.insert(octo);

        let mut expect4 = Delta::default();
        expect4.insert("!");

        let expected = [expect1, expect2, expect3, expect4];
        let attributes = [
            Attributes::default(),
            Attributes::default(),
            align,
            Attributes::default(),
        ];

        let p = |delta: &Delta, attr: &Attributes, line: usize| -> bool {
            //log::debug!( "line {:?} delta: {:?}", line, delta);
            //log::debug!( "line {:?} attribs {:?}",line, attr);
            assert_eq!(delta, expected.get(line).unwrap());
            assert_eq!(attr, attributes.get(line).unwrap());
            return true;
        };
        a.each_line(p, None)?;
        Ok(())
    }

    #[test]
    fn helper_eachline_trailing_newline_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.insert("Hello\nWorld!\n");

        let mut expect1 = Delta::default();
        expect1.insert("Hello");

        let mut expect2 = Delta::default();
        expect2.insert("World!");

        //No third line since that is empty !!
        // let expect3 = Delta::default();
        //
        let expected = [expect1, expect2];

        let p = |delta: &Delta, _attr: &Attributes, line: usize| -> bool {
            //log::debug!( "line {:?} delta: {:?}", line, delta);
            //log::debug!( "line {:?} attribs {:?}",line, attr);
            assert_eq!(delta, expected.get(line).unwrap());
            return true;
        };
        a.each_line(p, None)?;
        Ok(())
    }

    #[test]
    fn helper_eachline_non_document_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.retain(1);
        a.delete(2);

        let p = |_delta: &Delta, _attr: &Attributes, _line: usize| -> bool {
            //log::debug!( "line {:?} delta: {:?}", line, delta);
            //log::debug!( "line {:?} attribs {:?}",line, attr);
            //hey we should never have called the predicate !!
            assert_eq!(true, false);
            return true;
        };
        a.each_line(p, None)?;
        Ok(())
    }

    #[test]
    fn helper_eachline_early_return_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.insert("Hello\nNew\nWorld");

        let mut expect1 = Delta::default();
        expect1.insert("Hello");

        let mut expect2 = Delta::default();
        expect2.insert("New");

        let mut expect3 = Delta::default();
        expect3.insert("World");

        let expected = [expect1, expect2, expect3];

        let p = |delta: &Delta, _attr: &Attributes, line: usize| -> bool {
            //log::debug!( "line {:?} delta: {:?}", line, delta);
            //log::debug!( "line {:?} attribs {:?}",line, attr);
            assert_eq!(delta, expected.get(line).unwrap());
            return true;
        };
        a.each_line(p, None)?;
        Ok(())
    }

    #[test]
    fn helper_iteration_passes() {
        let mut img = OpsMap::default();
        img.insert("image", true);

        let mut a = Delta::default();
        a.insert("Hello");
        a.insert(img);
        a.insert("World!");

        let mut e1 = Delta::default();
        e1.insert("Hello");
        let mut e2 = Delta::default();
        e2.insert("World!");
        let f = |delta: &DeltaOperation, _index: usize| -> bool {
            //log::debug!( "line {:?} delta: {:?}", index, delta);
            return delta.insert_value().is_string();
        };
        let r = a.filter(f);
        assert_eq!(r.len(), 2);
        assert_eq!(r.get(0).unwrap(), e1.get(0).unwrap());
        assert_eq!(r.get(1).unwrap(), e2.get(0).unwrap());
        //dbg!(r);
    }

    #[test]
    fn helper_doc_length_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.insert_attr("AB", bold);
        delta.insert(1);

        assert_eq!(delta.delta_length(), 3);
    }

    #[test]
    fn helper_doc_length_mixed_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut bold_null = Attributes::default();
        bold_null.insert("bold", AttrVal::Null);

        let mut delta = Delta::default();
        delta.insert_attr("AB", bold);
        delta.insert(1);
        delta.retain_attr(2, bold_null);
        delta.delete(1);

        assert_eq!(delta.delta_length(), 6);
    }

    #[test]
    fn helper_doc_change_length_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut bold_null = Attributes::default();
        bold_null.insert("bold", AttrVal::Null);

        let mut delta = Delta::default();
        delta.insert_attr("AB", bold);
        delta.retain_attr(2, bold_null);
        delta.delete(1);

        assert_eq!(delta.document_length(), 1);
    }

    #[test]
    fn helper_slice_tart_and_end_chop_passes() {
        let mut delta = Delta::default();
        delta.insert("0123456789");

        let slc = delta.slice(2, 7);

        let mut expected = Delta::default();
        expected.insert("23456");

        assert_eq!(slc, expected);
    }

    #[test]
    fn helper_slice_start_and_multiple_chop_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.insert_attr("0123", bold.clone());
        delta.insert("4567");
        let slc = delta.slice(3, 5);

        let mut expected = Delta::default();
        expected.insert_attr("3", bold);
        expected.insert("4");

        assert_eq!(slc, expected);
    }

    #[test]
    fn helper_slice_start_and_end_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.retain(2);
        delta.insert_attr("A", bold.clone());
        delta.insert("B");
        let slc = delta.slice(2, 3);

        let mut expected = Delta::default();
        expected.insert_attr("A", bold);

        assert_eq!(slc, expected);
    }

    #[test]
    fn helper_slice_split_ops_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.insert_attr("AB", bold.clone());
        delta.insert("C");
        let slc = delta.slice(1, 2);

        let mut expected = Delta::default();
        expected.insert_attr("B", bold);

        assert_eq!(slc, expected);
    }

    #[test]
    fn helper_slice_ops_multiple_times_passes() {
        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.insert_attr("ABC", bold.clone());
        delta.insert("D");

        let slc = delta.slice(1, 2);

        let mut expected = Delta::default();
        expected.insert_attr("B", bold);

        assert_eq!(slc, expected);
    }
}
