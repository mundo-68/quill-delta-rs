# Quill-delta-rs 

This is a RUST implementation of the well-known `Quill Delta` library.
You can find the original JavaScript library at
[quill](https://github.com/quilljs/delta)

The functionality provided by this package is:
- Read `Delta` documents from string. Implemented as a feature,
- Edit the Delta document using operational transform commands,
- Write the `Delta` document to a `json`-formatted string.

The library provided is completely free of any formatting limitations.
So there are no checks that the attributes provided on a `Delta operation`
are actually valid.

Documentation:
 - User documentation: https://mundo-68.github.io/quill-delta-rs/delta/
 - Design documentation: [ ] todo ...

## Usage

```rust
fn main() {
    // create a delta and insert a string.
    let mut delta = Delta::default();
    delta.insert("Test");

    // create a link, and insert in to the delta document
    let mut attr = Attributes::default();
    attr.insert("src", "http://quilljs.com/image.png");
    delta.insert_attr( "link", attr);

    // Translate the delta document to a json string, and back.
    let json = serde_json::to_string(&delta).unwrap();
    let delta2: Delta = serde_json::from_str(&json).unwrap();
}


```

## About the diffs packages
The crate `diffs` was developed by Pierre-Étienne Meunier <pe@pijul.org>.
It is published on a private repository which can be publicly accessed, but not through `Cargo`. 
I put a copy of the `diffs` crate into this one for convenience.

As shown in the `Cargo.toml` file, the `diffs` crate, the license is MIT/Apache-2.0.<br>
All credits, etc., etc. for this crate should go to the original author.


## License

Licensed under either of
* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Testing

There is no HTML `DOM` relation in `quill-delta-rs`. <br>
For testing use:

```text
cargo test
```

# Other implementations
- Typescript( = Original quill) -> https://github.com/quilljs/delta
- Python -> https://github.com/forgeworks/quill-delta-python
- Dart -> https://github.com/pulyaevskiy/quill-delta-dart
