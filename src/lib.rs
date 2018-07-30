//!
//! Lazy concatenation of `String`s, `Vec`s and any other concatenable data structures. 
//! 
//! A `LazyConcat` owns a base structure and keeps track of fragments which can be 
//! either borrowed or owned. The fragments are never actually concatenated until the 
//! structure is normalized with `normalize()` or partially with `normalize_to_len()`.
//! 
//! When borrowing a slice, it is possible to normalize only the minimum number of 
//! fragments required for the slice. Various iterators over `String` or `Vec` are 
//! supported without the need to normalize first. 
//! 
//! # Examples
//! 
//! ```
//! # use lazy_concat::LazyConcat;
//! let mut lz = LazyConcat::new(Vec::new())
//!     // Concatenating owned values
//!     .concat(vec![0, 1, 2, 3, 4])
//!     // And borrowed values
//!     .concat(&[5, 6, 7, 8][..])
//!     .concat(&[9, 10][..]);
//! // This is possible without the above slice being concatenated
//! for i in lz.iter() {
//!     println!("i = {}", i);
//! }
//! // Actually concatenate enough values so that up to 6 elements can be sliced
//! lz.normalize_to_len(6);
//! let slice: &[i32] = lz.get_slice(2..6);
//! assert_eq!(&[2, 3, 4, 5], slice);
//! ```
//! 
//! 
use std::{
    fmt::{self, Debug, Formatter},
    borrow::{Cow, Borrow},
    ops::{RangeBounds, Bound},
    mem,
    iter,
};

pub(crate) mod concat;
pub(crate) mod length;
pub(crate) mod sliceable;

pub use length::Length;
pub use concat::Concat;
pub use sliceable::Sliceable;

pub struct LazyConcat<'a, T, B> 
where 
    B: ?Sized + 'a + ToOwned
{
    root: T,
    fragments: Vec<Fragment<'a, B>>,
}

pub(crate) enum Fragment<'a, B> 
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
            Fragment::Value(b) => b,
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
    // TODO Remove Default requirement. This could be done by putting root in an Option perhaps.
    T: Concat<Cow<'a, B>> + Borrow<B> + Default + Length,
    B: ToOwned<Owned = T> + ?Sized + Length,
{
    /// Construct a new `LazyConcat`. The initial value should be an owned value, such as a `Vec` or 
    /// a `String`. This can be empty but it doesn't have to be.
    pub fn new(initial: T) -> Self {
        LazyConcat { root: initial, fragments: Vec::new() }
    }
    
    /// Construct a new `LazyConcat`, but preallocate the vector of fragments with the expected number
    /// of fragments, so that won't need to be reallocated as fragments are added.
    pub fn expecting_num_fragments(initial: T, n: usize) -> Self {
        LazyConcat { root: initial, fragments: Vec::with_capacity(n) }
    }

    /// Fully normalize the collection by concatenating every fragament onto the base.
    pub fn normalize(&mut self) {
        self.normalize_range(..);
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
        if self.root.len() >= len {
            Some(self.root.len())
        } else if let Some(num) = self.fragments
            .iter()
            .scan(0, |total, ref fragment| {
                *total += fragment.len();
                Some(*total)
            })
            .position(|s| s >= len - self.root.len()) 
        {
            self.normalize_range(..=num);
            Some(self.root.len())
        } else {
            None
        }
    }

    /// The amount of data (in bytes) that has already been normalized. This is the maximum length 
    /// of a slice that can be taken without first calling `normalize()` or `normalize_to_len()`.
    #[inline]
    pub fn get_normalized_len(&self) -> usize {
        self.root.len()
    }

    /// Checks if any normalization is required before taking a slice
    /// 
    /// # Examples
    /// ```
    /// # use lazy_concat::LazyConcat;
    /// # let mut lz = LazyConcat::new(Vec::new())
    /// #   .concat(&[0,1,2,3,4,5,6][..]);
    /// if lz.slice_needs_normalization(1..3) {
    ///     lz.normalize_to_len(4);
    /// }
    /// let slice = lz.get_slice(1..3);
    /// ```
    /// 
    #[inline]
    pub fn slice_needs_normalization<R: RangeBounds<usize>>(&mut self, range: R) -> bool {
        match range.end_bound() {
            Bound::Unbounded => self.fragments.len() == 0,
            Bound::Excluded(&n) => self.root.len() < n,
            Bound::Included(&n) => self.root.len() <= n,
        }
    }

    /// Get a slice from the normalized data. Before calling this method you should check that the size of
    /// the normalized data is sufficient to be able to support this slice and, if necessary normalizing 
    /// the data to the required size using `normalize_to_len()`.
    /// # Panics
    /// Panics when the range falls outside the size of the owned data.
    pub fn get_slice<R: RangeBounds<usize>>(&self, range: R) -> &B
    where 
        T: Sliceable<Slice = B>
    {
        self.root.get_slice(range)
    }

    fn fragments_iter(&self) -> impl Iterator<Item = &B>
    where 
        T: Sliceable<Slice = B>
    {
        iter::once(self.root.get_slice(..))
            .chain(self.fragments.iter()
            .map(Fragment::borrow))
    }

    /// Consume the LazyConcat, concatenate all of the fragments and return the owned, fully normalized data.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use lazy_concat::LazyConcat;
    /// let lz = LazyConcat::new(String::from("abc"))
    ///     .concat("def")
    ///     .concat("ghi");
    /// 
    /// let result: String = lz.done();
    /// assert_eq!("abcdefghi", result);
    /// ```
    #[inline]
    pub fn done(mut self) -> T {
        self.normalize();
        self.root
    }

    /// Lazily concatenate an owned or borrowed fragment of data. No data will be moved or copied until the
    /// next time that `normalize()` or `normalize_to_len()` is called.
    /// 
    /// This is the same as `concat_in_place` except that it consumes and returns self, allowing for 
    /// method chaining.
    pub fn concat<F: Into<Cow<'a, B>>>(mut self, fragment: F) -> Self {
        self.concat_in_place(fragment);
        self
    }
    
    /// Lazily concatenate an owned or borrowed fragment of data. No data will be moved or copied until the
    /// next time that `normalize()` or `normalize_to_len()` is called.
    pub fn concat_in_place<F: Into<Cow<'a, B>>>(&mut self, fragment: F) {
        self.fragments.push(Fragment::Value(fragment.into()));
    }
}

impl<'a> LazyConcat<'a, String, str> {
    /// Creates an iterator over the `char`s of the String and any concatenated fragments.
    /// No normalization needs to be done for this to work.
    pub fn chars<'b>(&'b self) -> impl Iterator<Item = char> + 'b {
        self.fragments_iter()
            .flat_map(|fragment| {
                fragment.chars()
            })
    }

    /// Creates an iterator over the raw bytes of the String and any concatenated fragments.
    /// No normalization needs to be done for this to work.
    pub fn bytes<'b>(&'b self) -> impl Iterator<Item = u8> + 'b {
        self.fragments_iter()
            .flat_map(|fragment| {
                fragment.bytes()
            })
    }
}

impl<'a, I: Clone> LazyConcat<'a, Vec<I>, [I]> {
    /// Creates an iterator over references to items of a Vec and any concatenated fragments.
    /// No normalization needs to be done for this to work.
    pub fn iter(&self) -> impl Iterator<Item = &I> {
        self.fragments_iter()
            .flat_map(|slice| {
                slice.iter()
            })
    }

    /// Creates an iterator over the owned items of a Vec and any concatenated fragments.
    /// No normalization needs to be done for this to work.
    pub fn into_iter<'b>(&'b self) -> impl Iterator<Item = I> + 'b {
        self.fragments_iter()
            .flat_map(|fragment| {
                fragment.iter().cloned()
            })
    }
}


impl<'a, T, B> Debug for LazyConcat<'a, T, B> 
where
    T: Concat<Cow<'a, B>> + Borrow<B> + Debug,
    B: ToOwned<Owned = T> + ?Sized + Debug,
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
    use super::LazyConcat;

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
    fn string_iter_bytes() {
        let a = "形聲";
        let b = "網";
        let c = "网";
        let lz = LazyConcat::new(String::new())
            .concat(a)
            .concat(b.to_owned())
            .concat(c);

        let chars: Vec<u8> = lz.bytes().collect();
        assert_eq!(vec![229, 189, 162, 232, 129, 178, 231, 182, 178, 231, 189, 145], chars);
        // should not have normalized it
        assert_eq!("LazyConcat { \"\", \"形聲\", \"網\", \"网\" }", format!("{:?}", lz));
    }

    #[test] 
    fn vec_iter() {
        let a = vec![1,2,3];
        let b = vec![4,5];
        let c = vec![6,7,8];
        let d = vec![9];
        let lz = LazyConcat::new(Vec::new())
            .concat(&a)
            .concat(&b)
            .concat(&c)
            .concat(&d);

        let v: Vec<u32> = lz.iter().cloned().collect();
        assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], v);
        // should not have normalized it
        assert_eq!("LazyConcat { [], [1, 2, 3], [4, 5], [6, 7, 8], [9] }", format!("{:?}", lz));
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
            .concat(&c);

        {
            assert_eq!(0, lz.get_normalized_len());
            assert_eq!(true, lz.slice_needs_normalization(1..4));
            assert_eq!("LazyConcat { [], [1, 2, 3], [4, 5], [6, 7, 8] }", format!("{:?}", lz));

            lz.normalize_to_len(4);
            assert_eq!(false, lz.slice_needs_normalization(1..4));
            assert_eq!("LazyConcat { [1, 2, 3, 4, 5], [6, 7, 8] }", format!("{:?}", lz));
            let slice = lz.get_slice(1..4);
            assert_eq!(vec![2,3,4], slice);
            assert_eq!(5, lz.get_normalized_len());
        }
        lz = lz.concat(&d);
        assert_eq!("LazyConcat { [1, 2, 3, 4, 5], [6, 7, 8], [9] }", format!("{:?}", &lz));
        {
            let slice = lz.get_slice(2 .. 3);
            assert_eq!(vec![3], slice);
        }
    }
}
