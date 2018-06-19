use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};
use ::concat::Concat;

pub struct Normalized;
pub struct Fragmented;

pub struct LazyConcat<T, B, F = Fragmented> {
    root: T,
    fragments: Vec<B>,
    phantom: PhantomData<F>,
}

pub type NormalizedConcat<T, B> = LazyConcat<T, B, Normalized>;

impl<T, B> LazyConcat<T, B, Fragmented> 
where
    T: Concat<B, Output = T>
{
    pub fn new(initial: T) -> LazyConcat<T, B, Normalized> {
        LazyConcat { root: initial, fragments: Vec::new(), phantom: PhantomData }
    }
    
    pub fn expecting_num_fragments(initial: T, n: usize) -> LazyConcat<T, B, Normalized> {
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