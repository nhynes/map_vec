# map_vec

**The Map API backed by a Vec**

[`map_vec::Map`](https://docs.rs/map_vec) is a data structure with the interface of [`std::collections::HashMap`](https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html).

It's primarily useful when you care about constant factors or prefer determinism to speed.
Please refer to the [docs for `HashMap`](https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html) for details and examples of the Map API, as supported by [`map_vec::Map`](https://docs.rs/map_vec).

## Example

```rust
fn main() {
  let mut map = map_vec::Map::new();
  map.insert("hello".to_string(), "world".to_string());
  map.entry("hello".to_string()).and_modify(|mut v| v.push_str("!"));
  assert_eq!(map.get("hello").map(String::as_str), Some("world!"))
}
```
