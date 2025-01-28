
# reusing-vec

This crate provides a simple wrapper over [`Vec`] that allows elements to be reused without dropping them.

This is useful when you have a `Vec` of nested structures, and you want to manipulate them without the overhead of freeing memory to then reallocate it.

For example, pushing and popping from a `Vec<String>` in a loop would lead to allocs and frees as the [`String`] types were allocated and then dropped, only to be allocated again.  With a [`ReusingVec`], the string can reuse the allocation because the string is never dropped.

The interface is a more limited form of the `Vec` interface, without methods and implementations that allow for the re-acquisition of element ownership.

Additionally there is the [`ReusingQueue`], for the same purpose, but when a [`pop_front`] method is needed.

## Usage
```rust
use reusing_vec::ReusingVec;

let text = r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Sed rutrum mauris orci, ut iaculis purus cursus nec.
Praesent ultrices, lectus nec pulvinar scelerisque, ex leo imperdiet enim, volutpat dapibus nibh eros eu dolor.
Mauris nunc neque, faucibus et elit at, placerat feugiat ex.
Morbi aliquet velit in nunc congue, sed tempor augue accumsan.
"#;

let mut words_vec: ReusingVec::<String> = ReusingVec::new();

//After the `ReusingVec` has grown enough, no more allocations are needed
for line in text.lines() {
    words_vec.clear();
    for word in line.split(' ') {
        if word.len() > 0 {
            // Prepare a new empty vector element, to receive the data 
            words_vec.push_empty();

            // Extend the String in-place, with the uppercase chars.
            // String doesn't need to do any allocation after the initial buffer
            // is big enough.
            let uc_char_iter = word.chars().map(|c| c.to_uppercase()).flatten();
            words_vec.last_mut().unwrap().extend(uc_char_iter);
        }
    }

    words_vec.sort();

    // Do something here that requires a sorted uppercase vector of words...
}
```

[`ReusingVec`]: https://docs.rs/reusing-vec/latest/reusing_vec/struct.ReusingVec.html
[`ReusingQueue`]: https://docs.rs/reusing-vec/latest/reusing_vec/struct.ReusingQueue.html
[`pop_front`]: https://docs.rs/reusing-vec/latest/reusing_vec/struct.ReusingQueue.html#method.pop_front
[`String`]: https://doc.rust-lang.org/std/string/struct.String.html
[`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
