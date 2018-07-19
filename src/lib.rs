use std::{
    fmt::{self, Debug, Formatter},
    borrow::{Cow, Borrow},
    ops::RangeBounds,
    mem,
};

pub mod concat;
pub mod length;

use length::Length;
use concat::Concat;


pub struct LazyConcat<'a, T, B> 
where 
    B: ?Sized + 'a + ToOwned
{
    root: T,
    fragments: Vec<Fragment<'a, B>>,
}

pub enum Fragment<'a, B> 
where 
    B: ?Sized + 'a + ToOwned
{
    Value(Cow<'a, B>),
}

impl<'a, B: 'a> Fragment<'a, B> 
where 
    B: ToOwned + ?Sized,
{
    #[inline]
    fn get(self) -> Cow<'a, B> {
        match self {
            Fragment::Value(b) => b
        }
    }

    #[inline]
    fn borrow(&self) -> &B {
        match self {
            Fragment::Value(ref b) => b.borrow()
        }
    }

    fn len(&self) -> usize 
    where
        B: Length,
    {
        self.borrow().len()
    }
}

impl<'a, B> Debug for Fragment<'a, B> 
where 
    B: ToOwned + 'a,
    B: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        self.borrow().fmt(f)
    }
}

impl<'a, T, B> LazyConcat<'a, T, B> 
where
    T: Concat<Cow<'a, B>> + Borrow<B> + Default + Length,
    B: ToOwned<Owned = T> + ?Sized + Length,
{
    pub fn new(initial: T) -> Self {
        LazyConcat { root: initial, fragments: Vec::new() }
    }
    
    pub fn expecting_num_fragments(initial: T, n: usize) -> Self {
        LazyConcat { root: initial, fragments: Vec::with_capacity(n) }
    }

    pub fn normalize(mut self) -> Self {
        self.normalize_range(..);
        self
    }

    fn normalize_range<R: RangeBounds<usize>>(&mut self, range: R) {
        let fragments = self.fragments.drain(range);
        let root = mem::replace(&mut self.root, Default::default());
        self.root = fragments.fold(root, |agg, frag| agg.concat(frag.get()));
    }

    /// Normalize at least `len` elements and return the number of elements that were actually normalized.
    /// This could fail if there are not enough fragments to make up the required length, in which case
    /// `None` is returned and no work is done.
    pub fn normalize_to_len(&mut self, len: usize) -> Option<usize> {
        if let Some(num) = self.fragments
            .iter()
            .scan(0, |total, ref fragment| {
                *total += fragment.len();
                Some(*total)
            })
            .position(|s| s >= len) 
        {
            self.normalize_range(..=num);
            Some(self.root.len())
        } else {
            None
        }
    }

    #[inline]
    pub fn done(self) -> T {
        self.normalize().root
    }

    pub fn concat<F: Into<Cow<'a, B>>>(mut self, fragment: F) -> Self {
        self.fragments.push(Fragment::Value(fragment.into()));
        self
    }
}

impl<'a, T, B> Debug for LazyConcat<'a, T, B> 
where
    T: Concat<Cow<'a, B>> + Borrow<B>,
    B: ToOwned<Owned = T> + ?Sized,
    T: Debug,
    B: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "LazyConcat {{ {:?}", &self.root)?;
        for frag in &self.fragments {
            write!(f, ", {:?}", &frag)?;
        }
        f.write_str(" }")?;
        Ok(())
    }
}

impl<'a, T, B> From<T> for LazyConcat<'a, T, B> 
where
    T: Concat<Cow<'a, B>> + Borrow<B> + Default + Length,
    B: ToOwned<Owned = T> + ?Sized + Length,
{
    fn from(base: T) -> Self {
        LazyConcat::new(base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let a = "hel";
        let b = "lo the";
        let c = "re!";
        let lz = LazyConcat::new(String::new())
            .concat(a)
            .concat(b.to_owned())
            .concat(c);

        assert_eq!("LazyConcat { \"\", \"hel\", \"lo the\", \"re!\" }", format!("{:?}", lz));

        let lz = lz.normalize();
        assert_eq!("LazyConcat { \"hello there!\" }", format!("{:?}", lz));

        let res = lz.done();
        assert_eq!("hello there!", res);
    }

    #[test] 
    fn normalize_to_len() {
        let a = "hel";
        let b = "lo the";
        let c = "re!";
        let mut lz = LazyConcat::new(String::new())
            .concat(a)
            .concat(b.to_owned())
            .concat(c);
        let res = lz.normalize_to_len(6);
        assert_eq!(Some(9), res);
        assert_eq!("LazyConcat { \"hello the\", \"re!\" }", format!("{:?}", lz));
    }
}
