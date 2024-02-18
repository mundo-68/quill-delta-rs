use anyhow::Result;
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::optransform::OpTransform;
use delta::types::attr_val::AttrVal;

#[test]
fn compose_insert_insert_passes() -> Result<()> {
    let mut a = Delta::default();
    a.insert("A");

    let mut b = Delta::default();
    b.insert("B");

    let mut expected = Delta::default();
    expected.insert("B");
    expected.insert("A");

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_insert_retain_passes() -> Result<()> {
    let mut a = Delta::default();
    a.insert("A");

    let mut attr = Attributes::default();
    attr.insert("bold", true);
    attr.insert("color", "red");
    attr.insert("font", AttrVal::Null);

    let mut b = Delta::default();
    b.retain_attr(1, attr);

    let mut attr = Attributes::default();
    attr.insert("bold", true);
    attr.insert("color", "red");

    let mut expected = Delta::default();
    expected.insert_attr("A", attr);

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_insert_delete_passes() -> Result<()> {
    let mut a = Delta::default();
    a.insert("A");

    let mut b = Delta::default();
    b.delete(1);

    let expected = Delta::default();
    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_delete_insert_passes() -> Result<()> {
    let mut a = Delta::default();
    a.delete(1);

    let mut b = Delta::default();
    b.insert("B");

    let mut expected = Delta::default();
    expected.insert("B");
    expected.delete(1);

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_delete_retain_passes() -> Result<()> {
    let mut a = Delta::default();
    a.delete(1);

    let mut attr = Attributes::default();
    attr.insert("bold", true);
    attr.insert("color", "red");

    let mut b = Delta::default();
    b.retain_attr(1, attr.clone());

    let mut expected = Delta::default();
    expected.delete(1);
    expected.retain_attr(1, attr);

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_delete_delete_passes() -> Result<()> {
    let mut a = Delta::default();
    a.delete(1);

    let mut b = Delta::default();
    b.delete(1);

    let mut expected = Delta::default();
    expected.delete(2);

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_retain_insert_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("color", "blue");

    let mut a = Delta::default();
    a.retain_attr(1, attr.clone());

    let mut b = Delta::default();
    b.insert("B");

    let mut expected = Delta::default();
    expected.insert("B");
    expected.retain_attr(1, attr);

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_retain_retain_passes() -> Result<()> {
    let mut attr1 = Attributes::default();
    attr1.insert("color", "blue");

    let mut a = Delta::default();
    a.retain_attr(1, attr1.clone());

    let mut attr2 = Attributes::default();
    attr2.insert("bold", true);
    attr2.insert("color", "red");
    attr2.insert("font", AttrVal::Null);

    let mut b = Delta::default();
    b.retain_attr(1, attr2.clone());

    let mut expected = Delta::default();
    expected.retain_attr(1, attr2);

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_retain_delete_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("color", "blue");

    let mut a = Delta::default();
    a.retain_attr(1, attr);

    let mut b = Delta::default();
    b.delete(1);

    let mut expected = Delta::default();
    expected.delete(1);

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_insert_in_middle_of_text_passes() -> Result<()> {
    let mut a = Delta::default();
    a.insert("Hello");

    let mut b = Delta::default();
    b.retain(3);
    b.insert("X");

    let mut expected = Delta::default();
    expected.insert("HelXlo");

    let r = a.compose(&b)?;
    assert_eq!(r, expected);
    Ok(())
}

#[test]
fn compose_insert_and_delete_ordering_passes() -> Result<()> {
    let mut a = Delta::default();
    a.insert("Hello");

    let mut b = Delta::default();
    b.insert("Hello");

    let mut insert_first = Delta::default();
    insert_first.retain(3);
    insert_first.insert("X");
    insert_first.delete(1);

    let mut delete_first = Delta::default();
    delete_first.retain(3);
    delete_first.delete(1);
    delete_first.insert("X");

    let mut expected = Delta::default();
    expected.insert("HelXo");

    let r = a.compose(&delete_first)?;
    assert_eq!(&r, &expected);

    let r = b.compose(&insert_first)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_insert_embed_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("src", "http://quilljs.com/image.png");
    let mut a = Delta::default();
    a.insert_attr(1, attr);

    let mut attr = Attributes::default();
    attr.insert("alt", "logo");
    let mut b = Delta::default();
    b.retain_attr(1, attr);

    let mut attr = Attributes::default();
    attr.insert("src", "http://quilljs.com/image.png");
    attr.insert("alt", "logo");
    let mut expected = Delta::default();
    expected.insert_attr(1, attr);

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_delete_entire_text_passes() -> Result<()> {
    let mut a = Delta::default();
    a.retain(4);
    a.insert("Hello");

    let mut b = Delta::default();
    b.delete(9);

    let mut expected = Delta::default();
    expected.delete(4);

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_retain_more_than_length_of_text_text_passes() -> Result<()> {
    let mut a = Delta::default();
    a.insert("Hello");

    let mut b = Delta::default();
    b.retain(10);

    let mut expected = Delta::default();
    expected.insert("Hello");

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_retain_empty_embed_function_passes() -> Result<()> {
    let mut a = Delta::default();
    a.insert(1);

    let mut b = Delta::default();
    b.retain(1);

    let mut expected = Delta::default();
    expected.insert(1);

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_remove_all_attributes_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("bold", true);
    let mut a = Delta::default();
    a.insert_attr("A", attr);

    let mut attr = Attributes::default();
    attr.insert("bold", AttrVal::Null);
    let mut b = Delta::default();
    b.retain_attr(1, attr);

    let mut expected = Delta::default();
    expected.insert("A");

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_remove_all_embed_attributes_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let mut a = Delta::default();
    a.insert_attr(2, attr);

    let mut attr = Attributes::default();
    attr.insert("bold", AttrVal::Null);
    let mut b = Delta::default();
    b.retain_attr(1, attr);

    let mut expected = Delta::default();
    expected.insert(2);

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_immutability_passes() -> Result<()> {
    let mut attr1 = Attributes::default();
    attr1.insert("bold", true);

    let mut attr2 = Attributes::default();
    attr2.insert("bold", true);

    let mut a1 = Delta::default();
    a1.insert_attr("Test", attr1.clone());

    let mut a2 = Delta::default();
    a2.insert_attr("Test", attr2.clone());

    let mut attr = Attributes::default();
    attr.insert("color", "red");
    let mut b1 = Delta::default();
    b1.retain_attr(1, attr.clone());
    b1.delete(2);

    let mut b2 = Delta::default();
    b2.retain_attr(1, attr);
    b2.delete(2);

    let mut attr = Attributes::default();
    attr.insert("bold", true);
    attr.insert("color", "red");
    let mut expected = Delta::default();
    expected.insert_attr("T", attr);
    expected.insert_attr("t", attr1.clone());

    let r = a1.compose(&b1)?;
    assert_eq!(&r, &expected);
    assert_eq!(a1, a2);
    assert_eq!(b1, b2);
    assert_eq!(attr1, attr2);
    Ok(())
}

#[test]
fn compose_retain_start_optimization_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("bold", true);
    let mut a = Delta::default();
    a.insert_attr("A", attr.clone());
    a.insert("B");
    a.insert_attr("C", attr.clone());
    a.delete(1);

    let mut b = Delta::default();
    b.retain(3);
    b.insert("D");

    let mut expected = Delta::default();
    expected.insert_attr("A", attr.clone());
    expected.insert("B");
    expected.insert_attr("C", attr.clone());
    expected.insert("D");
    expected.delete(1);

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_retain_start_optimization_split_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("bold", true);
    let mut a = Delta::default();
    a.insert_attr("A", attr.clone());
    a.insert("B");
    a.insert_attr("C", attr.clone());
    a.retain(5);
    a.delete(1);

    let mut b = Delta::default();
    b.retain(4);
    b.insert("D");

    let mut expected = Delta::default();
    expected.insert_attr("A", attr.clone());
    expected.insert("B");
    expected.insert_attr("C", attr.clone());
    expected.retain(1);
    expected.insert("D");
    expected.retain(4);
    expected.delete(1);

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_retain_end_optimization_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("bold", true);
    let mut a = Delta::default();
    a.insert_attr("A", attr.clone());
    a.insert("B");
    a.insert_attr("C", attr.clone());

    let mut b = Delta::default();
    b.delete(1);

    let mut expected = Delta::default();
    expected.insert("B");
    expected.insert_attr("C", attr.clone());

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}

#[test]
fn compose_retain_end_optimization_join_passes() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("bold", true);
    let mut a = Delta::default();
    a.insert_attr("A", attr.clone());
    a.insert("B");
    a.insert_attr("C", attr.clone());
    a.insert("D");
    a.insert_attr("E", attr.clone());
    a.insert("F");

    let mut b = Delta::default();
    b.retain(1);
    b.delete(1);

    let mut expected = Delta::default();
    expected.insert_attr("AC", attr.clone());
    expected.insert("D");
    expected.insert_attr("E", attr.clone());
    expected.insert("F");

    let r = a.compose(&b)?;
    assert_eq!(&r, &expected);
    Ok(())
}
