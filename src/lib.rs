#![allow(dead_code)]
#![allow(unused_imports)]
// #![feature(macro_at_most_once_rep)]

use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::borrow::{Borrow, Cow};
use std::mem;

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
    // TODO: Normalize first - will require interior mutability
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


impl<'a> Borrow<str> for LazyConcat<'a, String> {
    fn borrow(&self) -> &str {
        ""
    }
}

// impl<'a, T> ToOwned for LazyConcat<'a, T>
// where
//     T: ToOwned + 'a + ?Sized,
//     T::Owned: Concat<T> + Default,
// {
//     type Owned = T::Owned;
//     fn to_owned(mut self) -> Self::Owned {
//         self.normalize();
//         if self.root.is_some () {
//             self.root.unwrap()
//         } else {
//             T::Owned::default()
//         }
//     }
// }


#[cfg(test)]
mod tests {
    #![allow(unused_variables)]
    use super::*;

    #[test]
    fn test_1() {
        let a = "hel";
        let b = "l";
        let c = "o the";
        let d = "re!";
        let mut lz: LazyConcat<str> = LazyConcat::new();
        lz.concat(a);
        lz.concat(b);

        println!("lz debug: {:?}", lz);
        println!("lz display: {}", lz);

        lz.normalize();

        lz.concat(c);
        lz.concat(d);


        println!("2 lz debug: {:?}", lz);
        println!("2 lz display: {}", lz);


        lz.normalize();
        println!("3 lz debug: {:?}", lz);
        println!("3 lz display: {}", lz);
        assert!(false);
    }
}