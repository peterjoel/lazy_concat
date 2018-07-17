use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};
use ::concat::Concat;

pub struct Normalized;
pub struct Fragmented;

pub struct LazyConcat<T, B, F = Fragmented> {
    root: T,
    fragments: Vec<Fragment<B>>,
    phantom: PhantomData<F>,
}

pub enum Fragment<B> {
    Value(B),
    Boxed(Box<B>),
    // TODO: This should be FnOnce. But can't be yet until (for example) 
    // https://github.com/rust-lang/rust/issues/28796 is resolved.
    Thunk(Box<Fn() -> B>),
}

impl<B> Fragment<B> {
    #[inline]
    fn get(self) -> B {
        match self {
            Fragment::Value(x) => x,
            Fragment::Boxed(b) => *b,
            Fragment::Thunk(thunk) => thunk(),
        }
    }
}

impl<B: Debug> Debug for Fragment<B> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Fragment::Value(x) => x.fmt(f),
            Fragment::Boxed(b) => b.fmt(f),
            Fragment::Thunk(_) => f.write_str("<fn>"),
        }
    }
}

pub type NormalizedConcat<T, B> = LazyConcat<T, B, Normalized>;

impl<T, B> LazyConcat<T, B, Fragmented> 
where
    T: Concat<B>,
{
    pub fn new(initial: T) -> LazyConcat<T, B, Normalized> {
        LazyConcat { root: initial, fragments: Vec::new(), phantom: PhantomData }
    }
    
    pub fn expecting_num_fragments(initial: T, n: usize) -> LazyConcat<T, B, Normalized> {
        LazyConcat { root: initial, fragments: Vec::with_capacity(n), phantom: PhantomData }
    }

    pub fn normalize(mut self) -> LazyConcat<T, B, Normalized> {
        {
            let fragments = self.fragments.drain(..);
            self.root = fragments.fold(self.root, |agg, frag| agg.concat(frag.get()));
        }
        self.change_type()
    }
}

impl<T, B> LazyConcat<T, B, Normalized> {

    #[inline]
    pub fn done(self) -> T {
        self.root
    }

    #[inline]
    pub fn get(&self) -> &T {
        &self.root
    }
}

impl<T, B, F> LazyConcat<T, B, F> 
where
    T: Concat<B>,
{
    #[inline]
    fn change_type<X>(self) -> LazyConcat<T, B, X> {
        LazyConcat { root: self.root, fragments: self.fragments, phantom: PhantomData }
    } 

    pub fn concat(mut self, fragment: B) -> LazyConcat<T, B, Fragmented> {
        self.fragments.push(Fragment::Value(fragment));
        self.change_type()
    }

    pub fn concat_boxed(mut self, fragment: Box<B>) -> LazyConcat<T, B, Fragmented> {
        self.fragments.push(Fragment::Boxed(fragment));
        self.change_type()
    }

    pub fn concat_later<M>(mut self, m: M) -> LazyConcat<T, B, Fragmented> 
    where
        M: Fn() -> B + 'static,
    {
        self.fragments.push(Fragment::Thunk(Box::new(m)));
        self.change_type()
    }
}

impl<T, B, F> Debug for LazyConcat<T, B, F> 
where
    T: Concat<B> + Debug,
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

impl<T, B> Display for LazyConcat<T, B, Normalized>
where
    T: Concat<B> + Display,
    B: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.get())?;
        Ok(())
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
            .concat(b)
            .concat(c);

        assert_eq!("LazyConcat { , \"\", \"hel\", \"lo the\", \"re!\" }", format!("{:?}", lz));

        let lz = lz.normalize();
        assert_eq!("hello there!", format!("{}", lz));

        let res = lz.done();
        assert_eq!("hello there!", res);
    }

    #[test]
    fn concat_box_and_closure() {
        let lz = LazyConcat::new(String::from("Hello"))
            .concat_boxed(Box::new(" "))
            .concat_later(|| "Peter" )
            .concat(" and good morning");
        assert_eq!("Hello Peter and good morning", lz.normalize().done());
    }
}
