pub trait Length 
{
    fn len(&self) -> usize;
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
