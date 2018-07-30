# lazy_concat
Lazy concatenation to `String` or `Vec`, supporting iteration and slices.

## Usage

```Toml
[dependencies]
lazy_concat = "0.1.1"
```

```Rust
extern crate lazy_concat;
use lazy_concat::LazyConcat;

let mut lazy_string = LazyConcat::new(String::new())
    // No allocations happen here
    .concat("Hello")
    .concat(" ")
    .concat("there!");

// Iteration works without any new allocation
for byte in lazy_string.bytes() {
    println!("byte = {:?}", byte);
}
// This extra block scope is not required with #[feature(nll)] (non-linear lifetimes).
{
    // Before taking a slice, make sure the required range is already concatenated
    if lazy_string.slice_needs_normalization(1..4) {
        lazy_string.normalize_to_len(4);
    }
    let slice = lazy_string.get_slice(1..4);
    assert_eq!("ell", slice);
}
// Finally allocate and concatenate the remainder of the string
let string: String = lazy_string.done();
assert_eq!("Hello there!", string);
```