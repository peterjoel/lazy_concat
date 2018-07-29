/// 
/// A trait for types whose values have a length, in bytes.
/// 

pub trait Length 
{
    /// The size of the object in bytes
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Length for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> Length for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Length for String {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Length for str {
    fn len(&self) -> usize {
        self.len()
    }
}
