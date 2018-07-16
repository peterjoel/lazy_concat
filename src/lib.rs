pub mod consuming;
pub mod concat;

mod mutating {
    use ::consuming::LazyConcat as ConsumingLazyCat;
    use ::consuming::{Normalized, Fragmented};
    use ::concat::Concat;
    use std::mem;

    pub enum LazyConcat<T, B> {
        Normalized(ConsumingLazyCat<T, B, Normalized>),
        Fragmented(ConsumingLazyCat<T, B, Fragmented>),
    }

    impl<T, B> LazyConcat<T, B> 
    where
        T: Concat<B> + Default,
    {

        pub fn new(initial: T) -> LazyConcat<T, B> {
            LazyConcat::Normalized(ConsumingLazyCat::new(initial))
        }
        
        pub fn expecting_num_fragments(initial: T, n: usize) -> LazyConcat<T, B> {
            LazyConcat::Normalized(ConsumingLazyCat::expecting_num_fragments(initial, n))
        }

        pub fn concat(&mut self, other: B) {
            let mutated = match mem::replace(self, LazyConcat::new(T::default())) {
                LazyConcat::Normalized(lz) => LazyConcat::Fragmented(lz.concat(other)),
                LazyConcat::Fragmented(lz) => LazyConcat::Fragmented(lz.concat(other)),
            };
            mem::replace(self, mutated);
        }
        
        pub fn normalize(&mut self) {
            if let LazyConcat::Normalized(_) = self {
                return;
            }
            let mutated = match mem::replace(self, LazyConcat::new(T::default())) {
                LazyConcat::Normalized(_) => unreachable!(),
                LazyConcat::Fragmented(lz) => LazyConcat::Normalized(lz.normalize()),
            };
            mem::replace(self, mutated);
        }
    }
}
