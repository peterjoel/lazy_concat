use ::length::Length;

pub trait Sliceable: Length {
    type Slice: ?Sized;
    // TODO: Replace this with SliceIndex, RangeBounds when stable
    fn get_slice(&self, from: Option<usize>, to: Option<usize>) -> &Self::Slice;
}

fn get_bounds<T>(target: &T, from: Option<usize>, to: Option<usize>) -> (usize, usize)
where
    T: Length,
{
    let len = target.len();
    let start = from.unwrap_or(0);
    let end = to.unwrap_or(len);
    assert!(start <= end);
    assert!(end <= len);
    (start, end)
}

impl<T> Sliceable for Vec<T> {
    type Slice = [T];
    fn get_slice(&self, from: Option<usize>, to: Option<usize>) -> &Self::Slice {
        let (start, end) = get_bounds(self, from, to);
        &self[start .. end]
    }
}

impl Sliceable for String {
    type Slice = str;
    fn get_slice(&self, from: Option<usize>, to: Option<usize>) -> &Self::Slice {
        let (start, end) = get_bounds(self, from, to);
        &self[start .. end]
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vec() {
        let vec = vec![1,2,3,4,5];
        let slice: &[u32] = vec.get_slice(Some(2), None);
        assert_eq!(vec![3,4,5], slice);
    }

    #[test]
    fn test_string() {
        let string = String::from("hello");
        let slice: &str = string.get_slice(None, Some(3));
        assert_eq!("hel", slice);
    }
}