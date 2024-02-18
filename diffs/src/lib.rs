//! Various diff (longest common subsequence) algorithms, used in
//! practice:
//!
//! - Myers' diff, in time O((N+M)D) and space O(N+M), where N and M
//! are the sizes of the old and new version, respectively. See [the
//! original article by Eugene
//! W. Myers](http://www.xmailserver.org/diff2.pdf).
//!
//! - Patience diff, in time O(N log N + M log M + (N+M)D), and space
//! O(N+M), which tends to give more human-readable outputs. See [Bram
//! Cohen's blog post describing
//! it](https://bramcohen.livejournal.com/73318.html).

pub mod replace;
pub use replace::*;
/// Myers' diff algorithm
pub mod myers;
/// Patience diff algorithm
pub mod patience;

pub use myers::diff;

// #[cfg(test)]
// mod test;

#[allow(unused_variables)]
/// A trait for reacting to an edit script from the "old" version to
/// the "new" version.
pub trait Diff: Sized {
    type Error;
    /// Called when lines with indices `old` (in the old version) and
    /// `new` (in the new version) start an section equal in both
    /// versions, of length `len`.
    fn equal(&mut self, old: usize, new: usize, len: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    /// Called when a section of length `len`, starting at `old`,
    /// needs to be deleted from the old version.
    fn delete(&mut self, old: usize, len: usize, new: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    /// Called when a section of the new version, of length `new_len`
    /// and starting at `new`, needs to be inserted at position `old'.
    fn insert(&mut self, old: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    /// Called when a section of the old version, starting at index
    /// `old` and of length `old_len`, needs to be replaced with a
    /// section of length `new_len`, starting at `new`, of the new
    /// version.
    fn replace(
        &mut self,
        old: usize,
        old_len: usize,
        new: usize,
        new_len: usize,
    ) -> Result<(), Self::Error> {
        self.delete(old, old_len, new)?;
        self.insert(old, new, new_len)
    }
    /// Always called at the end of the algorithm.
    fn finish(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, D: Diff + 'a> Diff for &'a mut D {
    type Error = D::Error;
    fn equal(&mut self, old: usize, new: usize, len: usize) -> Result<(), Self::Error> {
        (*self).equal(old, new, len)
    }
    fn delete(&mut self, old: usize, len: usize, new: usize) -> Result<(), Self::Error> {
        (*self).delete(old, len, new)
    }

    fn insert(&mut self, old: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        (*self).insert(old, new, new_len)
    }

    fn replace(
        &mut self,
        old: usize,
        old_len: usize,
        new: usize,
        new_len: usize,
    ) -> Result<(), Self::Error> {
        (*self).replace(old, old_len, new, new_len)
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        (*self).finish()
    }
}
