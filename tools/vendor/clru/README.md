# CLru

[![Actions](https://github.com/marmeladema/clru-rs/workflows/Rust/badge.svg)](https://github.com/marmeladema/clru-rs/actions)
[![Crate](https://img.shields.io/crates/v/clru)](https://crates.io/crates/clru)
[![Docs](https://docs.rs/clru/badge.svg)](https://docs.rs/clru)
[![License](https://img.shields.io/crates/l/clru)](LICENSE)

Another [LRU cache](https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_recently_used_(LRU)) implementation in rust.
It has two main characteristics that differentiates it from other implementation:

1. It is backed by a [HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html): it
   offers a O(1) time complexity (amortized average) for common operations like:
   * `get` / `get_mut`
   * `put` / `pop`
   * `peek` / `peek_mut`

2. It is a weighted cache: each key-value pair has a weight and the capacity serves as both as:
   * a limit to the number of elements
   * and as a limit to the total weight of its elements

   using the following formula:

   [`CLruCache::len`](https://docs.rs/clru/latest/clru/struct.CLruCache.html#method.len) + [`CLruCache::weight`](https://docs.rs/clru/latest/clru/struct.CLruCache.html#method.weight) <= [`CLruCache::capacity`](https://docs.rs/clru/latest/clru/struct.CLruCache.html#method.capacity)

Even though most operations don't depend on the number of elements in the cache,
[`CLruCache::put_with_weight`](https://docs.rs/clru/latest/clru/struct.CLruCache.html#method.put_with_weight) has a special behavior: because it needs to make room
for the new element, it will remove enough least recently used elements. In the worst
case, that will require to fully empty the cache. Additionally, if the weight of the
new element is too big, the insertion can fail.

For the common case of an LRU cache whose elements don't have a weight, a default
[`ZeroWeightScale`](https://docs.rs/clru/latest/clru/struct.ZeroWeightScale.html)
is provided and unlocks some useful APIs like:

* [`CLruCache::put`](https://docs.rs/clru/latest/clru/struct.CLruCache.html#method.put): an infallible insertion that will remove a maximum of 1 element.
* [`CLruCache::put_or_modify`](https://docs.rs/clru/latest/clru/struct.CLruCache.html#method.put_or_modify): a conditional insertion or modification flow similar to the entry API of [`HashMap`].
* [`CLruCache::try_put_or_modify`](https://docs.rs/clru/latest/clru/struct.CLruCache.html#method.try_put_or_modify): fallible version of `CLruCache::put_or_modify`.
* All APIs that allow to retrieve a mutable reference to a value (e.g.: [`CLruCache::get_mut`](https://docs.rs/clru/latest/clru/struct.CLruCache.html#method.get_mut)).

## Disclaimer

Most of the API, documentation, examples and tests have been heavily inspired by the [lru](https://github.com/jeromefroe/lru-rs) crate.
I want to thank [jeromefroe](https://github.com/jeromefroe/) for his work without which this crate would have probably never has been released.


## Differences with [lru](https://github.com/jeromefroe/lru-rs)

The main differences are:
* Smaller amount of unsafe code. Unsafe code is not bad in itself as long as it is thoroughly reviewed and understood but can be surprisingly hard to get right. Reducing the amount of unsafe code should hopefully reduce bugs or undefined behaviors.
* API closer to the standard [HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html) collection which allows to lookup with `Borrow`-ed version of the key.

## Example

Below are simple examples of how to instantiate and use this LRU cache.

### Using the default [`ZeroWeightScale`](https://docs.rs/clru/latest/clru/struct.ZeroWeightScale.html):

```rust

use std::num::NonZeroUsize;
use clru::CLruCache;

let mut cache = CLruCache::new(NonZeroUsize::new(2).unwrap());
cache.put("apple".to_string(), 3);
cache.put("banana".to_string(), 2);

assert_eq!(cache.get("apple"), Some(&3));
assert_eq!(cache.get("banana"), Some(&2));
assert!(cache.get("pear").is_none());

assert_eq!(cache.put("banana".to_string(), 4), Some(2));
assert_eq!(cache.put("pear".to_string(), 5), None);

assert_eq!(cache.get("pear"), Some(&5));
assert_eq!(cache.get("banana"), Some(&4));
assert!(cache.get("apple").is_none());

{
    let v = cache.get_mut("banana").unwrap();
    *v = 6;
}

assert_eq!(cache.get("banana"), Some(&6));
```

### Using a custom [`WeightScale`](https://docs.rs/clru/latest/clru/trait.WeightScale.html) implementation:

```rust

use std::num::NonZeroUsize;
use clru::{CLruCache, CLruCacheConfig, WeightScale};

struct CustomScale;

impl WeightScale<String, &str> for CustomScale {
    fn weight(&self, _key: &String, value: &&str) -> usize {
        value.len()
    }
}

let mut cache = CLruCache::with_config(
    CLruCacheConfig::new(NonZeroUsize::new(6).unwrap()).with_scale(CustomScale),
);

assert_eq!(cache.put_with_weight("apple".to_string(), "red").unwrap(), None);
assert_eq!(
    cache.put_with_weight("apple".to_string(), "green").unwrap(),
    Some("red")
);

assert_eq!(cache.len(), 1);
assert_eq!(cache.get("apple"), Some(&"green"));
```

## Tests

Each contribution is tested with regular compiler, miri, and 4 flavors of sanitizer (address, memory, thread and leak).
This should help catch bugs sooner than later.

## TODO

* improve documentation and add examples
