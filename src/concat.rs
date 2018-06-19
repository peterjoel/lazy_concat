use std::borrow::{Cow, Borrow};

pub trait Concat<T = Self>
where T: ?Sized,
{
    type Output;
    fn concat(self, other: T) -> Self::Output;
}

impl Concat for String {
    type Output = String;
    fn concat(mut self, other: Self) -> String {
        self.push_str(&other);
        self
    }
}

impl<'a> Concat<&'a str> for String {
    type Output = String;
    fn concat(mut self, other: &str) -> String {
        self.push_str(&other);
        self
    }
}

impl<'a> Concat<&'a str> for &'a str {
    type Output = String;
    fn concat(self, other: &str) -> String {
        let mut owned = self.to_owned();
        owned.push_str(&other);
        owned
    }
}

impl<'a> Concat<&'a str> for Cow<'a, str> {
    type Output = Cow<'a, str>;
    fn concat(self, other: &'a str) -> Cow<'a, str> {
        let owned = self.into_owned();
        Cow::Owned(owned.concat(other))
    }
}

impl<'a> Concat<String> for Cow<'a, str> {
    type Output = Cow<'a, str>;
    fn concat(self, other: String) -> Cow<'a, str> {
        let owned = self.into_owned();
        Cow::Owned(owned.concat(other.borrow()))
    }
}

impl<'a> Concat<Cow<'a, str>> for Cow<'a, str> {
    type Output = Cow<'a, str>;
    fn concat(self, other: Cow<'a, str>) -> Cow<'a, str> {
        let owned = self.into_owned();
        Cow::Owned(owned.concat(other.borrow()))
    }
}

impl<'a, T> Concat<&'a [T]> for Vec<T> 
    where 
        T: Clone 
{
    type Output = Vec<T>;
    fn concat(mut self, other: &'a [T]) -> Vec<T> {
        self.extend_from_slice(other);
        self
    }
}