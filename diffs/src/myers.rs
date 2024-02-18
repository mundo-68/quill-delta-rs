use std::cmp::{max, min};
use std::ops::Index;
use Diff;

fn modulo(a: isize, b: usize) -> usize {
    let b = b as isize;
    (((a % b) + b) % b) as usize
}

#[test]
fn test_modulo() {
    assert_eq!(modulo(-11, 10), 9);
    assert_eq!(modulo(23, 7), 2);
    assert_eq!(modulo(-12, 6), 0);
}

/// Myers' diff algorithm. Diff `e`, between indices `e0` (included)
/// and `e1` (excluded), on the one hand, and `f`, between indices
/// `f0` (included)` and `f1` (excluded), on the other hand.
pub fn diff<S: Index<usize> + ?Sized, T: Index<usize> + ?Sized, D: Diff>(
    d: &mut D,
    e: &S,
    e0: usize,
    e1: usize,
    f: &T,
    f0: usize,
    f1: usize,
) -> Result<(), D::Error>
where
    T::Output: PartialEq<S::Output>,
{
    diff_offsets(d, e, e0, e1, f, f0, f1)?;
    d.finish()
}

pub(crate) fn diff_offsets<D: Diff + ?Sized, S: Index<usize> + ?Sized, T: Index<usize> + ?Sized>(
    diff: &mut D,
    e: &S,
    i: usize,
    i_: usize,
    f: &T,
    j: usize,
    j_: usize,
) -> Result<(), D::Error>
where
    T::Output: PartialEq<S::Output>,
{
    if i_ > i && j_ > j {
        let n = i_ - i;
        let m = j_ - j;
        let l = (n + m) as isize;
        let z = (2 * min(n, m) + 2) as usize;
        let w = n as isize - m as isize;
        let mut g = vec![0; z as usize];
        let mut p = vec![0; z as usize];
        for h in 0..=(l / 2 + l % 2) {
            macro_rules! search {
                ($e: expr, $c: expr, $d: expr) => {
                    let (k0, k1) = {
                        let (m, n) = (m as isize, n as isize);
                        (-(h - 2*max(0, h - m)), h-2*max(0, h-n)+1)
                    };
                    for k in (k0..k1).step_by(2) {
                        let mut a: usize = if k == -h || k != h && $c[modulo(k-1, z)] < $c[modulo(k+1, z)] {
                            $c[modulo(k+1, z)]
                        } else {
                            $c[modulo(k-1, z)] + 1
                        };
                        let mut b = (a as isize - k) as usize;
                        let (s, t) = (a, b);
                        while a < n && b < m && {
                            let (e_i, f_i) = if $e { (a, b) } else { (n - a - 1, m - b - 1) };
                            f[j + f_i] == e[i + e_i]
                        } {
                            a += 1;
                            b += 1;
                        }
                        $c[modulo(k, z)] = a;
                        let bound = if $e { h-1 } else { h };
                        if (l%2 == 1) == $e
                            && w-k >= -bound && w-k <= bound
                            && $c[modulo(k, z)]+$d[modulo(w-k, z)] >= n
                        {
                            let (x, y, u, v) = if $e {
                                (s, t, a, b)
                            } else {
                                (n-a, m-b, n-s, m-t)
                            };
                            if h + bound > 1 || (x != u && y != v) {
                                diff_offsets(diff, e, i, i+x, f, j, j+y)?;
                                if x != u {
                                    diff.equal(i + x, j + y, u-x)?;
                                }
                                diff_offsets(diff, e, i+u, i_, f, j+v, j_)?;
                                return Ok(())
                            } else if m > n {
                                diff.equal(i, j, n)?;
                                diff.insert(i+n, j+n, m-n)?;
                                return Ok(())
                            } else if m < n {
                                diff.equal(i, j, m)?;
                                diff.delete(i+m, n-m, j+m)?;
                                return Ok(())
                            } else {
                                return Ok(())
                            }
                        }
                    }
                }
            }
            search!(true, g, p);
            search!(false, p, g);
        }
    } else if i_ > i {
        diff.delete(i, i_ - i, j)?
    } else if j_ > j {
        diff.insert(i, j, j_ - j)?
    }
    Ok(())
}
