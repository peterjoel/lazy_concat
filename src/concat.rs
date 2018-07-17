use std::borrow::{Cow, Borrow};
use std::ffi::{OsStr, OsString};
use std::ops::Deref;

pub trait Concat<T = Self>
where
    T: ?Sized,
{
    fn concat(self, other: T) -> Self;
}

impl<'a, D: Deref<Target = str> + 'a> Concat<D> for String {
    fn concat(mut self, other: D) -> Self {
        self.push_str(&other);
        self
    }
}

impl<'a, D: Deref<Target = str> + 'a> Concat<D> for Cow<'a, str> {
    fn concat(self, other: D) -> Cow<'a, str> {
        let owned = self.into_owned();
        Cow::Owned(owned.concat(&*other))
    }
}

impl<'a, D: Deref<Target = OsStr> + 'a> Concat<D> for OsString {
        fn concat(mut self, other: D) -> OsString {
        self.push(&*other);
        self
    }
}

impl<'a, T> Concat<&'a [T]> for Vec<T> 
where
    T: Clone,
{
    fn concat(mut self, other: &'a [T]) -> Vec<T> {
        self.extend_from_slice(other);
        self
    }
}

impl<T> Concat<T> for Vec<T> {
    fn concat(mut self, item: T) -> Vec<T> {
        self.push(item);
        self
    }
}

impl<'a, T: Clone> Concat<&'a T> for Vec<T> {
    fn concat(mut self, item: &'a T) -> Vec<T> {
        self.push(item.clone());
        self
    }
}

impl<T> Concat<Vec<T>> for Vec<T> {
    fn concat(mut self, other: Vec<T>) -> Vec<T> {
        self.extend(other);
        self
    }
}

macro_rules! vec_concat_array {
    ($($n: expr),*) => {
        $(
            impl<'a, T> Concat<[T; $n]> for Vec<T> 
            where
                T: Clone,
            {
                fn concat(mut self, other: [T; $n]) -> Vec<T> {
                    self.extend_from_slice(other.borrow());
                    self
                }
            }
        )*
    }
}

vec_concat_array!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_concat_string() {
        let s = String::from("abc");
        let res: String = s.concat(String::from("123"));
        assert_eq!(res, "abc123");
    }

    #[test]
    fn string_concat_str() {
        let s = String::from("abc");
        let res: String = s.concat("123");
        assert_eq!(res, "abc123");
    }

    #[test]
    fn cow_concat_str() {
        let s = Cow::from("abc");
        let res: Cow<str> = s.concat("123");
        assert_eq!(res, "abc123");
    }

    #[test]
    fn cow_concat_string() {
        let s = Cow::from("abc");
        let res: Cow<str> = s.concat(String::from("123"));
        assert_eq!(res, "abc123");
    }

    #[test]
    fn cow_concat_cow() {
        let s = Cow::from("abc");
        let res: Cow<str> = s.concat(Cow::from("123"));
        assert_eq!(res, "abc123");
    }

    #[test]
    fn osstring_concat_osstr() {
        let s = OsString::from("abc");
        let to_append = OsString::from("123");
        let res: OsString = s.concat(&to_append[..]);
        assert_eq!(res, OsString::from("abc123"));
    }
    
    #[test]
    fn vec_concat_slice() {
        let s = vec![1, 2, 3];
        let to_append = vec![4, 5];
        let res: Vec<u32> = s.concat(&to_append[..]);
        assert_eq!(res, vec![1, 2, 3, 4, 5]);
    }    
    
    #[test]
    fn vec_concat_element() {
        let s = vec![1, 2, 3];
        let res: Vec<u32> = s.concat(4);
        assert_eq!(res, vec![1, 2, 3, 4]);
    }  
    
    #[test]
    fn vec_concat_element_ref() {
        let s = vec![1, 2, 3];
        let res: Vec<u32> = s.concat(&4);
        assert_eq!(res, vec![1, 2, 3, 4]);
    }  

    #[test]
    fn vec_concat_vec() {
        let s = vec![1, 2, 3];
        let to_append = vec![4, 5];
        let res: Vec<u32> = s.concat(to_append);
        assert_eq!(res, vec![1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn vec_concat_array_1() {
        let s = vec![1, 2, 3];
        let to_append = [4];
        let res: Vec<u32> = s.concat(to_append);
        assert_eq!(res, vec![1, 2, 3, 4]);
    }    
    
    #[test]
    fn vec_concat_array_32() {
        let s = vec![1, 2, 3];
        let to_append = [4; 32];
        let res: Vec<u32> = s.concat(to_append);
        assert_eq!(res, vec![1, 2, 3, 4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4]);
    }
}
