use std::{
    fmt::{self, Debug, Display, Formatter},
    borrow::{Cow, Borrow},
    marker::PhantomData,
};

pub struct Normalized;
pub struct Fragmented;

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

pub struct LazyConcat<T, B, F = Fragmented> 
where
    T: Concat<B>
{
    root: T,
    fragments: Vec<B>,
    phantom: PhantomData<F>,
}

pub type NormalizedConcat<T, B> = LazyConcat<T, B, Normalized>;

impl<T, B> LazyConcat<T, B, Fragmented> 
where
    T: Concat<B, Output = T>
{
    pub fn new(initial: T) -> LazyConcat<T, B, Fragmented> {
        LazyConcat { root: initial, fragments: Vec::new(), phantom: PhantomData }
    }
    
    pub fn expecting_num_fragments(initial: T, n: usize) -> LazyConcat<T, B, Fragmented> {
        LazyConcat { root: initial, fragments: Vec::with_capacity(n), phantom: PhantomData }
    }
}

impl<T, B, F> LazyConcat<T, B, F> 
where
    T: Concat<B, Output = T>
{
    fn change_type<X>(self) -> LazyConcat<T, B, X> {
        LazyConcat { root: self.root, fragments: self.fragments, phantom: PhantomData }
    } 

    pub fn normalize(mut self) -> LazyConcat<T, B, Normalized> {
        {
            let fragments = self.fragments.drain(..);
            self.root = fragments.fold(self.root, |agg, frag| agg.concat(frag));
        }
        self.change_type()
    }

    pub fn concat(mut self, fragment: B) -> LazyConcat<T, B, Fragmented> {
        self.fragments.push(fragment);
        self.change_type()
    }
}

impl<T, B, F> Debug for LazyConcat<T, B, F> 
where
    T: Concat<B, Output = T> + Debug,
    B: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("LazyConcat { ")?;
        write!(f, ", {:?}", &self.root)?;
        for frag in &self.fragments {
            write!(f, ", {:?}", &frag)?;
        }
        f.write_str(" }")?;
        Ok(())
    }
}

impl<T, B, F> Display for LazyConcat<T, B, F>
where
    T: Concat<B, Output = T> + Display,
    B: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.root)?;
        for frag in &self.fragments {
            write!(f, "{}", &frag)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_variables)]
    use super::*;

    #[test]
    fn test_1() {
        let a = "hel";
        let b = "lo the";
        let c = "re!";
        let lz = LazyConcat::new(String::from(""))
            .concat(a)
            .concat(b)
            .concat(c);

        assert_eq!("hello there!", format!("{}", lz));

        let lz = lz.normalize();
        assert_eq!("hello there!", format!("{}", lz));
    }
}