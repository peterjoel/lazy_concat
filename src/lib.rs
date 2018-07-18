use std::{
    fmt::{self, Debug, Formatter},
    borrow::{Cow, Borrow},
};

pub mod concat;
use concat::Concat;


pub struct LazyConcat<'a, T, B> 
where 
    B: ?Sized + 'a + ToOwned
{
    root: T,
    fragments: Vec<Fragment<'a, B>>,
}

pub enum Fragment<'a, B> 
where 
    B: ?Sized + 'a + ToOwned
{
    Value(Cow<'a, B>),
}

impl<'a, B: 'a> Fragment<'a, B> 
where 
    B: ToOwned + ?Sized,
{
    #[inline]
    fn get(self) -> Cow<'a, B> {
        match self {
            Fragment::Value(b) => b
        }
    }

    #[inline]
    fn borrow(&self) -> &B {
        match self {
            Fragment::Value(ref b) => b.borrow()
        }
    }
}

impl<'a, B> Debug for Fragment<'a, B> 
where 
    B: ToOwned + 'a,
    B: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        self.borrow().fmt(f)
    }
}

impl<'a, T, B> LazyConcat<'a, T, B> 
where
    T: Concat<Cow<'a, B>> + Borrow<B>,
    B: ToOwned<Owned = T> + ?Sized,
{
    pub fn new(initial: T) -> LazyConcat<'a, T, B> {
        LazyConcat { root: initial, fragments: Vec::new() }
    }
    
    pub fn expecting_num_fragments(initial: T, n: usize) -> LazyConcat<'a, T, B> {
        LazyConcat { root: initial, fragments: Vec::with_capacity(n) }
    }

    pub fn normalize(mut self) -> LazyConcat<'a, T, B> {
        {
            let fragments = self.fragments.drain(..);
            self.root = fragments.fold(self.root, |agg, frag| agg.concat(frag.get()));
        }
        self
    }

    #[inline]
    pub fn done(self) -> T {
        self.normalize().root
    }

    #[inline]
    pub fn get(&self) -> &T {
        &self.root
    }

    pub fn concat_owned(mut self, fragment: B::Owned) -> Self {
        self.fragments.push(Fragment::Value(Cow::Owned(fragment)));
        self
    }

    pub fn concat_borrowed(mut self, fragment: &'a B) -> Self {
        self.fragments.push(Fragment::Value(Cow::Borrowed(fragment)));
        self
    }

    pub fn concat<F: Into<Cow<'a, B>>>(mut self, fragment: F) -> Self {
        self.fragments.push(Fragment::Value(fragment.into()));
        self
    }
}

impl<'a, T, B> Debug for LazyConcat<'a, T, B> 
where
    T: Concat<Cow<'a, B>> + Borrow<B>,
    B: ToOwned<Owned = T> + ?Sized,
    T: Debug,
    B: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "LazyConcat {{ {:?}", &self.root)?;
        for frag in &self.fragments {
            write!(f, ", {:?}", &frag)?;
        }
        f.write_str(" }")?;
        Ok(())
    }
}

impl<'a, T, B> From<T> for LazyConcat<'a, T, B> 
where
    T: Concat<Cow<'a, B>> + Borrow<B>,
    B: ToOwned<Owned = T> + ?Sized,
{
    fn from(base: T) -> Self {
        LazyConcat::new(base)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let a = "hel";
        let b = "lo the";
        let c = "re!";
        let lz = LazyConcat::new(String::new())
            .concat(a)
            .concat(b.to_owned())
            .concat(c);

        assert_eq!("LazyConcat { \"\", \"hel\", \"lo the\", \"re!\" }", format!("{:?}", lz));

        let lz = lz.normalize();
        assert_eq!("LazyConcat { \"hello there!\" }", format!("{:?}", lz));

        let res = lz.done();
        assert_eq!("hello there!", res);
    }
}
