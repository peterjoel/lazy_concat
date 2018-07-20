# lazy_concat
Lazy `String` and `Vec` concatenation with iteration and slices.

## Basic Usage

```Rust
extern crate lazy_concat;
use lazy_concat::LazyConcat;

let lazy_string: String = LazyConcat::new(String::new())
    // No allocations happen here
    .concat("Hello")
    .concat(" ")
    .concat("there!");

// Until here, when the String is constructed
let string: String = lazy_string.done()
```

## Iteration

```Rust
let raw: Vec<u32> = (0..100).collect();

let lazy_vec = LazyConcat::new(Vec::new())
    .concat(&raw[0..2])
    .concat(&raw[10..12]);

// Still no new allocation here
for i in lazy_vec.iter() {
    println!("i = {:?}", i);
}
// Outputs: 0, 1, 10, 11

// Actually allocate
let v: Vec<u32> = lazy_vec.done();
assert_eq!(vec![0, 1, 10, 11], v);
```
Or over `String`s:

```Rust
// Does not need to allocate all the concatenated &strs to do this
for c in lazy_string.chars() {
    println!("char = {:?}", ch);
}
```

You can also just force the allocation in the middle, as an intermediary step, using `normalize()`.

## Slices

When you need a slice, it will only copy the minimum number of fragments required to create the contiguous memory needed by the size of slice you requested:

```Rust
// Temporary syntax. It will implement ranges (e.g. `get_slice(..2)`) later
let slice: &[u32] = lazy_vec.get_slice(None, Some(2));
```