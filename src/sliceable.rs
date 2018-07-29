/// A trait for types that can have slices taken from them.

use ::length::Length;
use std::ops::{Bound, RangeBounds};

pub trait Sliceable: Length {
    type Slice: ?Sized;
    fn get_slice<R>(&self, range: R) -> &Self::Slice
    where 
        R: RangeBounds<usize>;
}

fn bounds<T, R>(target: &T, range: R) -> (usize, usize)
where
    T: Length,
    R: RangeBounds<usize>,
{
    let len = target.len();
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(n) => *n,
        _ => unreachable!(),
    };
    let end = match range.end_bound() {
        Bound::Unbounded => len,
        Bound::Included(n) => *n + 1,
        Bound::Excluded(n) => *n,
    };
    assert!(start <= end);
    println!("end = {}, len = {}, range = {:?}", end, len, range.end_bound());
    assert!(end <= len);
    (start, end)
}

impl<T> Sliceable for Vec<T> {
    type Slice = [T];
    fn get_slice<R>(&self, range: R) -> &Self::Slice 
    where
        R: RangeBounds<usize>
    {
        let (start, end) = bounds(self, range);
        &self[start .. end]
    }
}

impl Sliceable for String {
    type Slice = str;
    fn get_slice<R>(&self, range: R) -> &Self::Slice 
    where
        R: RangeBounds<usize>
    {
        let (start, end) = bounds(self, range);
        &self[start .. end]
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vec_bounded() {
        let vec = vec![0,1,2,3,4,5];
        let slice: &[u32] = vec.get_slice(1..3);
        assert_eq!(vec![1, 2], slice);
    }
        #[test]
    fn test_vec_unbounded() {
        let vec = vec![0,1,2,3,4,5];
        let slice: &[u32] = vec.get_slice(3..);
        assert_eq!(vec![3,4,5], slice);
    }

    #[test]
    fn test_string() {
        let string = String::from("hello");
        let slice: &str = string.get_slice(0 ..= 2);
        assert_eq!("hel", slice);
    }
}