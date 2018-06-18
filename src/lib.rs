use std::{
    fmt::{self, Debug, Display, Formatter},
    borrow::Cow,
    mem
};

pub trait Concat<T = Self>
where T: ?Sized,
{
    fn concat(self, other: &T) -> Self;
}

impl Concat for String {
    fn concat(mut self, other: &String) -> String {
        self.push_str(&other);
        self
    }
}

impl Concat<str> for String {
    fn concat(mut self, other: &str) -> String {
        self.push_str(&other);
        self
    }
}

impl<'a> Concat for Cow<'a, str> {
    fn concat(self, other: &Cow<'a, str>) -> Cow<'a, str> {
        let mut owned = self.into_owned();
        owned.push_str(&other);
        Cow::Owned(owned)
    }
}


pub struct LazyConcat<'a, T> 
where
    T: ToOwned + 'a + ?Sized,
{
    root: Option<T::Owned>,
    fragments: Vec<&'a T>,
}

impl<'a, T> LazyConcat<'a, T> 
where
    T: ToOwned + 'a + ?Sized,
    T::Owned: Concat<T>
{
    pub fn new() -> LazyConcat<'a, T> {
        LazyConcat { root: None, fragments: Vec::new() }
    }

    pub fn expecting_num_fragments(n: usize) -> LazyConcat<'a, T> {
        LazyConcat { root: None, fragments: Vec::with_capacity(n) }
    }

    pub fn normalize(&mut self) {
        let mut fragments = self.fragments.drain(..);
        let first = if let Some(normal) = mem::replace(&mut self.root, None) {
            normal
        } else {
            fragments.next().map(|fragment| fragment.to_owned()).unwrap()
        };
        let normal = fragments.fold(first, |agg, frag| agg.concat(&frag));
        mem::replace(&mut self.root, Some(normal));
    }

    pub fn concat<'b: 'a>(&mut self, fragment: &'b T) {
        self.fragments.push(fragment);
    }

    pub fn is_normal(&self) -> bool {
        self.fragments.len() == 0
    }

    pub fn get_normal(&mut self) -> &Option<T::Owned> {
        self.normalize();
        &self.root
   }
}

impl<'a, T> Debug for LazyConcat<'a, T> 
where
    T: ToOwned + ?Sized + Debug,
    T::Owned: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("LazyConcat { ")?;
        write!(f, ", {:?}", &self.root)?;
        for frag in self.fragments.iter() {
            write!(f, ", {:?}", &frag)?;
        }
        f.write_str(" }")?;
        Ok(())
    }
}

impl<'a, T> Display for LazyConcat<'a, T>
where
    T: ToOwned + ?Sized + Display,
    T::Owned: Display,
{
    // TODO: Normalize first - will require interior mutability or consume-self
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(root) = &self.root {
            write!(f, "{}", root)?;
        }
        for frag in self.fragments.iter() {
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
        let mut lz: LazyConcat<str> = LazyConcat::new();
        lz.concat(a);
        lz.concat(b);
        lz.concat(c);

        assert_eq!(false, lz.is_normal());
        assert_eq!("hello there!", format!("{}", lz));
        assert_eq!(false, lz.is_normal());

        lz.normalize();
        assert_eq!(true, lz.is_normal());
        assert_eq!("hello there!", format!("{}", lz));
    }
}