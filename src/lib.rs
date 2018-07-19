#![feature(collections_range)]
use std::{
    fmt::{self, Debug, Formatter},
    borrow::{Cow, Borrow},
    ops::RangeBounds,
    mem,
    iter,
};

pub mod concat;
pub mod length;
pub mod sliceable;

use length::Length;
use concat::Concat;
use sliceable::Sliceable;


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

    pub fn normalize(&mut self) {
        self.normalize_range(..);
    }

    fn normalize_range<R: RangeBounds<usize>>(&mut self, range: R) {
        let fragments = self.fragments.drain(range);
        // TODO Remove Default requirement. This could be done by putting root in an Option perhaps.
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

    // TODO replace from/to optionals with ranges: .., a.., ..b, a..=b etc
    pub fn get_slice(&mut self, from: Option<usize>, to: Option<usize>) -> &B
    where 
        T: Sliceable<Slice = B>
    {
        if let Some(n) = to {
            self.normalize_to_len(n).expect("Cannot make slice: out of bounds!");
        } else {
            self.normalize();
        }
        self.root.get_slice(from, to)
    }

    fn fragments_iter(&self) -> impl Iterator<Item = &B>
    where 
        T: Sliceable<Slice = B>
    {
        iter::once(self.root.get_slice(None, None))
            .chain(self.fragments.iter()
            .map(Fragment::borrow))
    }


    #[inline]
    pub fn done(mut self) -> T {
        self.normalize();
        self.root
    }

    pub fn concat<F: Into<Cow<'a, B>>>(mut self, fragment: F) -> Self {
        self.fragments.push(Fragment::Value(fragment.into()));
        self
    }
}

impl<'a> LazyConcat<'a, String, str> {
    pub fn chars<'b>(&'b self) -> impl Iterator<Item = char> + 'b {
        self.fragments_iter()
            .flat_map(|slice| {
                slice.chars()
            })
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
        let mut lz = LazyConcat::new(String::new())
            .concat(a)
            .concat(b.to_owned())
            .concat(c);

        assert_eq!("LazyConcat { \"\", \"hel\", \"lo the\", \"re!\" }", format!("{:?}", lz));

        lz.normalize();
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

    #[test] 
    fn string_iter_chars() {
        let a = "hel";
        let b = "lo the";
        let c = "re!";
        let lz = LazyConcat::new(String::new())
            .concat(a)
            .concat(b.to_owned())
            .concat(c);

        let chars: Vec<char> = lz.chars().collect();
        assert_eq!(vec!['h', 'e', 'l', 'l', 'o', ' ', 't', 'h', 'e', 'r', 'e', '!'], chars);
        // should not have normalized it
        assert_eq!("LazyConcat { \"\", \"hel\", \"lo the\", \"re!\" }", format!("{:?}", lz));
    }


    #[test] 
    fn normalize_slice() {
        let a = vec![1,2,3];
        let b = vec![4,5];
        let c = vec![6,7,8];
        let d = vec![9];
        let mut lz = LazyConcat::new(Vec::new())
            .concat(&a)
            .concat(&b)
            .concat(&c)
            .concat(&d);
        {
            let slice = lz.get_slice(Some(1), Some(4));
            assert_eq!(vec![2,3,4], slice);
        }
        assert_eq!("LazyConcat { [1, 2, 3, 4, 5], [6, 7, 8], [9] }", format!("{:?}", lz));
    }
}
