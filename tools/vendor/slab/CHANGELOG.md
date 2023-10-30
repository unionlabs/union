# 0.4.6 (April 2, 2022)

* Add `Slab::vacant_key` (#114)
* Fix stacked borrows violation in `Slab::get2_unchecked_mut` (#115)

# 0.4.5 (October 13, 2021)

* Add alternate debug output for listing items in the slab (#108)
* Fix typo in debug output of IntoIter (#109)
* Impl 'Clone' for 'Iter' (#110)

# 0.4.4 (August 06, 2021)

* Fix panic in `FromIterator` impl (#102)
* Fix compatibility with older clippy versions (#104)
* Add `try_remove` method (#89)
* Implement `ExactSizeIterator` and `FusedIterator` for iterators (#92)

# 0.4.3 (April 20, 2021)

* Add no_std support for Rust 1.36 and above (#71).
* Add `get2_mut` and `get2_unchecked_mut` methods (#65).
* Make `shrink_to_fit()` remove trailing vacant entries (#62).
* Implement `FromIterator<(usize, T)>` (#62).
* Implement `IntoIterator<Item = (usize, T)>` (#62).
* Provide `size_hint()` of the iterators (#62).
* Make all iterators reversible (#62).
* Add `key_of()` method (#61)
* Add `compact()` method (#60)
* Add support for serde (#85)

# 0.4.2 (January 11, 2019)

* Add `Slab::drain` (#56).

# 0.4.1 (July 15, 2018)

* Improve `reserve` and `reserve_exact` (#37).
* Implement `Default` for `Slab` (#43).
