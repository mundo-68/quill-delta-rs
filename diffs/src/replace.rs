use Diff;

/// A "differ" that combines deletions and insertions to give blocks
/// of maximal length, and replacements when appropriate.
pub struct Replace<D: Diff> {
    d: D,
    del: Option<(usize, usize, usize)>,
    ins: Option<(usize, usize, usize)>,
    eq: Option<(usize, usize, usize)>,
}

impl<D: Diff> Replace<D> {
    pub fn new(d: D) -> Self {
        Replace {
            d,
            del: None,
            ins: None,
            eq: None,
        }
    }
    pub fn into_inner(self) -> D {
        self.d
    }
}

impl<D: Diff> AsRef<D> for Replace<D> {
    fn as_ref(&self) -> &D {
        &self.d
    }
}

impl<D: Diff> AsMut<D> for Replace<D> {
    fn as_mut(&mut self) -> &mut D {
        &mut self.d
    }
}

impl<D: Diff> Diff for Replace<D> {
    type Error = D::Error;
    fn equal(&mut self, old: usize, new: usize, len: usize) -> Result<(), D::Error> {
        if let Some((old0, len0, new0)) = self.del.take() {
            if let Some((_, new1, new_len1)) = self.ins.take() {
                self.d.replace(old0, len0, new1, new_len1)?
            } else {
                self.d.delete(old0, len0, new0)?
            }
        } else if let Some((old0, new0, new_len0)) = self.ins.take() {
            self.d.insert(old0, new0, new_len0)?
        }

        if let Some((a, b, c)) = self.eq.take() {
            self.eq = Some((a, b, c + len))
        } else {
            self.eq = Some((old, new, len))
        }
        Ok(())
    }
    fn delete(&mut self, old: usize, len: usize, new: usize) -> Result<(), D::Error> {
        if let Some((a, b, c)) = self.eq.take() {
            self.d.equal(a, b, c)?
        }
        if let Some((old0, len0, new0)) = self.del.take() {
            assert_eq!(old, old0 + len0);
            self.del = Some((old0, len0 + len, new0))
        } else {
            self.del = Some((old, len, new))
        }
        Ok(())
    }

    fn insert(&mut self, old: usize, new: usize, new_len: usize) -> Result<(), D::Error> {
        if let Some((a, b, c)) = self.eq.take() {
            self.d.equal(a, b, c)?
        }
        if let Some((old1, new1, new_len1)) = self.ins.take() {
            assert_eq!(new1 + new_len1, new);
            self.ins = Some((old1, new1, new_len + new_len1))
        } else {
            self.ins = Some((old, new, new_len))
        }
        Ok(())
    }

    fn replace(
        &mut self,
        old: usize,
        old_len: usize,
        new: usize,
        new_len: usize,
    ) -> Result<(), D::Error> {
        if let Some((a, b, c)) = self.eq.take() {
            self.d.equal(a, b, c)?
        }
        self.d.replace(old, old_len, new, new_len)
    }

    fn finish(&mut self) -> Result<(), D::Error> {
        if let Some((a, b, c)) = self.eq.take() {
            self.d.equal(a, b, c)?
        }
        if let Some((old0, len0, new0)) = self.del.take() {
            if let Some((_, new1, new_len1)) = self.ins.take() {
                self.d.replace(old0, len0, new1, new_len1)?
            } else {
                self.d.delete(old0, len0, new0)?
            }
        } else if let Some((old0, new0, new_len0)) = self.ins.take() {
            self.d.insert(old0, new0, new_len0)?
        }
        self.d.finish()
    }
}

#[test]
fn myers() {
    use myers;
    let a: &[&str] = &[
        ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n",
        "a\n",
        "b\n",
        "c\n",
        "================================\n",
        "d\n",
        "e\n",
        "f\n",
        "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<\n",
    ];
    let b: &[&str] = &[
        ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n",
        "x\n",
        "b\n",
        "c\n",
        "================================\n",
        "y\n",
        "e\n",
        "f\n",
        "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<\n",
    ];

    struct D {}
    impl Diff for D {
        type Error = ();
        fn equal(&mut self, o: usize, n: usize, len: usize) -> Result<(), ()> {
            println!("equal {:?} {:?} {:?}", o, n, len);
            Ok(())
        }
        fn delete(&mut self, o: usize, len: usize, new: usize) -> Result<(), ()> {
            println!("delete {:?} {:?} {:?}", o, len, new);
            Ok(())
        }
        fn insert(&mut self, o: usize, n: usize, len: usize) -> Result<(), ()> {
            println!("insert {:?} {:?} {:?}", o, n, len);
            Ok(())
        }
        fn replace(&mut self, o: usize, l: usize, n: usize, nl: usize) -> Result<(), ()> {
            println!("replace {:?} {:?} {:?} {:?}", o, l, n, nl);
            Ok(())
        }
    }
    let mut d = Replace::new(D {});
    myers::diff(&mut d, a, 0, a.len(), b, 0, b.len()).unwrap();
}
