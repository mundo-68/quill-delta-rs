use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;
use {myers, Diff, Replace};

struct I<'a, S: 'a + Index<usize> + ?Sized> {
    p: &'a S,
    i: usize,
}

impl<'a, A: Index<usize> + 'a> std::fmt::Debug for I<'a, A>
where
    A::Output: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", &self.p[self.i])
    }
}

impl<'a, 'b, A: Index<usize> + 'b + ?Sized, B: Index<usize> + 'b + ?Sized> PartialEq<I<'a, A>>
    for I<'b, B>
where
    B::Output: PartialEq<A::Output>,
{
    fn eq(&self, b: &I<'a, A>) -> bool {
        self.p[self.i] == b.p[b.i]
    }
}

fn unique<A: Hash + Eq, S: Index<usize, Output = A> + ?Sized>(
    p: &S,
    e0: usize,
    e1: usize,
) -> Vec<I<S>> {
    let mut aa = HashMap::new();
    for i in e0..e1 {
        match aa.entry(&p[i]) {
            Entry::Vacant(e) => {
                e.insert(Some(i));
            }
            Entry::Occupied(mut e) => {
                let e = e.get_mut();
                if e.is_some() {
                    *e = None
                }
            }
        }
    }
    let mut v: Vec<_> = aa
        .into_iter()
        .filter_map(|(_, x)| x)
        .map(|i| I { p, i })
        .collect();
    v.sort_by(|a, b| a.i.cmp(&b.i));
    v
}

/// Patience diff algorithm.
pub fn diff<
    A: Hash + Eq,
    B: Hash + Eq + PartialEq<A>,
    S: Index<usize, Output = A> + ?Sized,
    T: Index<usize, Output = B> + ?Sized,
    D: Diff,
>(
    d: &mut D,
    e: &S,
    e0: usize,
    e1: usize,
    f: &T,
    f0: usize,
    f1: usize,
) -> Result<(), D::Error> {
    let au = unique(e, e0, e1);
    let bu = unique(f, f0, f1);

    struct Patience<
        'a,
        'b,
        'd,
        S: 'a + Index<usize> + ?Sized,
        T: 'b + Index<usize> + ?Sized,
        D: Diff + 'd,
    > {
        current_a: usize,
        current_b: usize,
        a1: usize,
        b1: usize,
        a: &'a S,
        b: &'b T,
        d: &'d mut D,
        au: &'a [I<'a, S>],
        bu: &'b [I<'b, T>],
    }
    impl<
            'a,
            'b,
            'd,
            S: 'a + Index<usize> + ?Sized,
            T: 'b + Index<usize> + ?Sized,
            D: Diff + 'd,
        > Diff for Patience<'a, 'b, 'd, S, T, D>
    where
        T::Output: PartialEq<S::Output>,
    {
        type Error = D::Error;
        fn equal(&mut self, old: usize, new: usize, len: usize) -> Result<(), D::Error> {
            // eprintln!("equal {:?} {:?} {:?}", old, new, len);
            for (old, new) in (old..old + len).zip(new..new + len) {
                let a0 = self.current_a;
                let b0 = self.current_b;
                while self.current_a < self.au[old].i
                    && self.current_b < self.bu[new].i
                    && self.b[self.current_b] == self.a[self.current_a]
                {
                    self.current_a += 1;
                    self.current_b += 1;
                }
                if self.current_a > a0 {
                    self.d.equal(a0, b0, self.current_a - a0)?
                }
                // let a = &self.a[self.current_a..self.au[old].i];
                // let b = &self.b[self.current_b..self.bu[new].i];
                // eprintln!("matching a: {:?} {:?}", self.current_a, self.au[old].i);
                // eprintln!("matching b: {:?} {:?}", self.current_b, self.bu[new].i);
                myers::diff_offsets(
                    self.d,
                    self.a,
                    self.current_a,
                    self.au[old].i,
                    self.b,
                    self.current_b,
                    self.bu[new].i,
                )?;
                self.current_a = self.au[old].i;
                self.current_b = self.bu[new].i;
            }
            Ok(())
        }
        /*
        fn insert(&mut self, old: usize, new: usize, len: usize) -> Result<(), D::Error> {
            eprintln!("insert {:?} {:?} {:?}", old, new, len);
            Ok(())
        }
        fn delete(&mut self, old: usize, len: usize) -> Result<(), D::Error> {
            eprintln!("delete {:?} {:?}", old, len);
            Ok(())
        }
        fn replace(
            &mut self,
            old: usize,
            len: usize,
            new: usize,
            new_len: usize,
        ) -> Result<(), D::Error> {
            eprintln!("replace {:?} {:?} {:?} {:?}", old, len, new, new_len);
            Ok(())
        }
        */
        fn finish(&mut self) -> Result<(), D::Error> {
            myers::diff(
                self.d,
                self.a,
                self.current_a,
                self.a1,
                self.b,
                self.current_b,
                self.b1,
            )
        }
    }
    let mut d = Replace::new(Patience {
        current_a: e0,
        current_b: f0,
        a: e,
        a1: e1,
        b: f,
        b1: f1,
        d,
        au: &au,
        bu: &bu,
    });
    myers::diff(&mut d, &au, 0, au.len(), &bu, 0, bu.len())?;
    Ok(())
}

#[test]
fn patience() {
    let a: &[usize] = &[11, 1, 2, 2, 3, 4, 4, 4, 5, 47, 19];
    let b: &[usize] = &[10, 1, 2, 2, 8, 9, 4, 4, 7, 47, 18];
    struct D(Vec<(usize, usize, usize, usize)>);
    impl Diff for D {
        type Error = ();
        fn delete(&mut self, o: usize, len: usize, new: usize) -> Result<(), ()> {
            self.0.push((o, len, new, 0));
            Ok(())
        }
        fn insert(&mut self, o: usize, n: usize, len: usize) -> Result<(), ()> {
            self.0.push((o, 0, n, len));
            Ok(())
        }
        fn replace(&mut self, o: usize, l: usize, n: usize, nl: usize) -> Result<(), ()> {
            self.0.push((o, l, n, nl));
            Ok(())
        }
    }
    let mut d = Replace::new(D(Vec::new()));
    diff(&mut d, a, 0, a.len(), b, 0, b.len()).unwrap();
    let d: D = d.into_inner();
    assert_eq!(
        d.0.as_slice(),
        &[(0, 1, 0, 1), (4, 2, 4, 2), (8, 1, 8, 1), (10, 1, 10, 1)]
    );
}
