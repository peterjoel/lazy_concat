use std::borrow::{Cow, Borrow};

pub trait Concat<T = Self>
where
    T: ?Sized,
{
    fn concat(self, other: T) -> Self;
}

impl Concat for String {
    fn concat(mut self, other: Self) -> Self {
        self.push_str(&other);
        self
    }
}

impl<'a> Concat<&'a str> for String {
    fn concat(mut self, other: &str) -> String {
        self.push_str(&other);
        self
    }
}

impl<'a> Concat<&'a str> for Cow<'a, str> {
    fn concat(self, other: &'a str) -> Cow<'a, str> {
        let owned = self.into_owned();
        Cow::Owned(owned.concat(other))
    }
}

impl<'a> Concat<String> for Cow<'a, str> {
    fn concat(self, other: String) -> Cow<'a, str> {
        let owned = self.into_owned();
        Cow::Owned(owned.concat(other.borrow()))
    }
}

impl<'a> Concat<Cow<'a, str>> for Cow<'a, str> {
    fn concat(self, other: Cow<'a, str>) -> Cow<'a, str> {
        let owned = self.into_owned();
        Cow::Owned(owned.concat(other.borrow()))
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

impl<T> Concat<Vec<T>> for Vec<T> {
    fn concat(mut self, other: Vec<T>) -> Vec<T> {
        self.extend(other);
        self
    }
}

// impl<'a, T> Concat<&'a Vec<T>> for Vec<T>
// where
//     T: Clone,
// {
//     fn concat(mut self, other: &Vec<T>) -> Vec<T> {
//         self.extend_from_slice(&other);
//         self
//     }
// }
