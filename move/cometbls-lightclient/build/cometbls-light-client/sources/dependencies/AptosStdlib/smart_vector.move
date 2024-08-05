module aptos_std::smart_vector {
    use std::error;
    use std::vector;
    use aptos_std::big_vector::{Self, BigVector};
    use aptos_std::math64::max;
    use aptos_std::type_info::size_of_val;
    use std::option::{Self, Option};

    /// Vector index is out of bounds
    const EINDEX_OUT_OF_BOUNDS: u64 = 1;
    /// Cannot destroy a non-empty vector
    const EVECTOR_NOT_EMPTY: u64 = 2;
    /// Cannot pop back from an empty vector
    const EVECTOR_EMPTY: u64 = 3;
    /// bucket_size cannot be 0
    const EZERO_BUCKET_SIZE: u64 = 4;
    /// The length of the smart vectors are not equal.
    const ESMART_VECTORS_LENGTH_MISMATCH: u64 = 0x20005;

    /// A Scalable vector implementation based on tables, Ts are grouped into buckets with `bucket_size`.
    /// The option wrapping BigVector saves space in the metadata associated with BigVector when smart_vector is
    /// so small that inline_vec vector can hold all the data.
    struct SmartVector<T> has store {
        inline_vec: vector<T>,
        big_vec: Option<BigVector<T>>,
        inline_capacity: Option<u64>,
        bucket_size: Option<u64>,
    }

    /// Regular Vector API

    /// Create an empty vector using default logic to estimate `inline_capacity` and `bucket_size`, which may be
    /// inaccurate.
    /// This is exactly the same as empty() but is more standardized as all other data structures have new().
    public fun new<T: store>(): SmartVector<T> {
        empty()
    }

    #[deprecated]
    /// Create an empty vector using default logic to estimate `inline_capacity` and `bucket_size`, which may be
    /// inaccurate.
    public fun empty<T: store>(): SmartVector<T> {
        SmartVector {
            inline_vec: vector[],
            big_vec: option::none(),
            inline_capacity: option::none(),
            bucket_size: option::none(),
        }
    }

    /// Create an empty vector with customized config.
    /// When inline_capacity = 0, SmartVector degrades to a wrapper of BigVector.
    public fun empty_with_config<T: store>(inline_capacity: u64, bucket_size: u64): SmartVector<T> {
        assert!(bucket_size > 0, error::invalid_argument(EZERO_BUCKET_SIZE));
        SmartVector {
            inline_vec: vector[],
            big_vec: option::none(),
            inline_capacity: option::some(inline_capacity),
            bucket_size: option::some(bucket_size),
        }
    }

    /// Create a vector of length 1 containing the passed in T.
    public fun singleton<T: store>(element: T): SmartVector<T> {
        let v = empty();
        push_back(&mut v, element);
        v
    }

    /// Destroy the vector `v`.
    /// Aborts if `v` is not empty.
    public fun destroy_empty<T>(v: SmartVector<T>) {
        assert!(is_empty(&v), error::invalid_argument(EVECTOR_NOT_EMPTY));
        let SmartVector { inline_vec, big_vec, inline_capacity: _, bucket_size: _ } = v;
        vector::destroy_empty(inline_vec);
        option::destroy_none(big_vec);
    }

    /// Destroy a vector completely when T has `drop`.
    public fun destroy<T: drop>(v: SmartVector<T>) {
        clear(&mut v);
        destroy_empty(v);
    }

    /// Clear a vector completely when T has `drop`.
    public fun clear<T: drop>(v: &mut SmartVector<T>) {
        v.inline_vec = vector[];
        if (option::is_some(&v.big_vec)) {
            big_vector::destroy(option::extract(&mut v.big_vec));
        }
    }

    /// Acquire an immutable reference to the `i`th T of the vector `v`.
    /// Aborts if `i` is out of bounds.
    public fun borrow<T>(v: &SmartVector<T>, i: u64): &T {
        assert!(i < length(v), error::invalid_argument(EINDEX_OUT_OF_BOUNDS));
        let inline_len = vector::length(&v.inline_vec);
        if (i < inline_len) {
            vector::borrow(&v.inline_vec, i)
        } else {
            big_vector::borrow(option::borrow(&v.big_vec), i - inline_len)
        }
    }

    /// Return a mutable reference to the `i`th T in the vector `v`.
    /// Aborts if `i` is out of bounds.
    public fun borrow_mut<T>(v: &mut SmartVector<T>, i: u64): &mut T {
        assert!(i < length(v), error::invalid_argument(EINDEX_OUT_OF_BOUNDS));
        let inline_len = vector::length(&v.inline_vec);
        if (i < inline_len) {
            vector::borrow_mut(&mut v.inline_vec, i)
        } else {
            big_vector::borrow_mut(option::borrow_mut(&mut v.big_vec), i - inline_len)
        }
    }

    /// Empty and destroy the other vector, and push each of the Ts in the other vector onto the lhs vector in the
    /// same order as they occurred in other.
    /// Disclaimer: This function may be costly. Use it at your own discretion.
    public fun append<T: store>(lhs: &mut SmartVector<T>, other: SmartVector<T>) {
        let other_len = length(&other);
        let half_other_len = other_len / 2;
        let i = 0;
        while (i < half_other_len) {
            push_back(lhs, swap_remove(&mut other, i));
            i = i + 1;
        };
        while (i < other_len) {
            push_back(lhs, pop_back(&mut other));
            i = i + 1;
        };
        destroy_empty(other);
    }

    /// Add multiple values to the vector at once.
    public fun add_all<T: store>(v: &mut SmartVector<T>, vals: vector<T>) {
        vector::for_each(vals, |val| { push_back(v, val); })
    }

    /// Convert a smart vector to a native vector, which is supposed to be called mostly by view functions to get an
    /// atomic view of the whole vector.
    /// Disclaimer: This function may be costly as the smart vector may be huge in size. Use it at your own discretion.
    public fun to_vector<T: store + copy>(v: &SmartVector<T>): vector<T> {
        let res = v.inline_vec;
        if (option::is_some(&v.big_vec)) {
            let big_vec = option::borrow(&v.big_vec);
            vector::append(&mut res, big_vector::to_vector(big_vec));
        };
        res
    }

    /// Add T `val` to the end of the vector `v`. It grows the buckets when the current buckets are full.
    /// This operation will cost more gas when it adds new bucket.
    public fun push_back<T: store>(v: &mut SmartVector<T>, val: T) {
        let len = length(v);
        let inline_len = vector::length(&v.inline_vec);
        if (len == inline_len) {
            let bucket_size = if (option::is_some(&v.inline_capacity)) {
                if (len < *option::borrow(&v.inline_capacity)) {
                    vector::push_back(&mut v.inline_vec, val);
                    return
                };
                *option::borrow(&v.bucket_size)
            } else {
                let val_size = size_of_val(&val);
                if (val_size * (inline_len + 1) < 150 /* magic number */) {
                    vector::push_back(&mut v.inline_vec, val);
                    return
                };
                let estimated_avg_size = max((size_of_val(&v.inline_vec) + val_size) / (inline_len + 1), 1);
                max(1024 /* free_write_quota */ / estimated_avg_size, 1)
            };
            option::fill(&mut v.big_vec, big_vector::empty(bucket_size));
        };
        big_vector::push_back(option::borrow_mut(&mut v.big_vec), val);
    }

    /// Pop an T from the end of vector `v`. It does shrink the buckets if they're empty.
    /// Aborts if `v` is empty.
    public fun pop_back<T>(v: &mut SmartVector<T>): T {
        assert!(!is_empty(v), error::invalid_state(EVECTOR_EMPTY));
        let big_vec_wrapper = &mut v.big_vec;
        if (option::is_some(big_vec_wrapper)) {
            let big_vec = option::extract(big_vec_wrapper);
            let val = big_vector::pop_back(&mut big_vec);
            if (big_vector::is_empty(&big_vec)) {
                big_vector::destroy_empty(big_vec)
            } else {
                option::fill(big_vec_wrapper, big_vec);
            };
            val
        } else {
            vector::pop_back(&mut v.inline_vec)
        }
    }

    /// Remove the T at index i in the vector v and return the owned value that was previously stored at i in v.
    /// All Ts occurring at indices greater than i will be shifted down by 1. Will abort if i is out of bounds.
    /// Disclaimer: This function may be costly. Use it at your own discretion.
    public fun remove<T>(v: &mut SmartVector<T>, i: u64): T {
        let len = length(v);
        assert!(i < len, error::invalid_argument(EINDEX_OUT_OF_BOUNDS));
        let inline_len = vector::length(&v.inline_vec);
        if (i < inline_len) {
            vector::remove(&mut v.inline_vec, i)
        } else {
            let big_vec_wrapper = &mut v.big_vec;
            let big_vec = option::extract(big_vec_wrapper);
            let val = big_vector::remove(&mut big_vec, i - inline_len);
            if (big_vector::is_empty(&big_vec)) {
                big_vector::destroy_empty(big_vec)
            } else {
                option::fill(big_vec_wrapper, big_vec);
            };
            val
        }
    }

    /// Swap the `i`th T of the vector `v` with the last T and then pop the vector.
    /// This is O(1), but does not preserve ordering of Ts in the vector.
    /// Aborts if `i` is out of bounds.
    public fun swap_remove<T>(v: &mut SmartVector<T>, i: u64): T {
        let len = length(v);
        assert!(i < len, error::invalid_argument(EINDEX_OUT_OF_BOUNDS));
        let inline_len = vector::length(&v.inline_vec);
        let big_vec_wrapper = &mut v.big_vec;
        let inline_vec = &mut v.inline_vec;
        if (i >= inline_len) {
            let big_vec = option::extract(big_vec_wrapper);
            let val = big_vector::swap_remove(&mut big_vec, i - inline_len);
            if (big_vector::is_empty(&big_vec)) {
                big_vector::destroy_empty(big_vec)
            } else {
                option::fill(big_vec_wrapper, big_vec);
            };
            val
        } else {
            if (inline_len < len) {
                let big_vec = option::extract(big_vec_wrapper);
                let last_from_big_vec = big_vector::pop_back(&mut big_vec);
                if (big_vector::is_empty(&big_vec)) {
                    big_vector::destroy_empty(big_vec)
                } else {
                    option::fill(big_vec_wrapper, big_vec);
                };
                vector::push_back(inline_vec, last_from_big_vec);
            };
            vector::swap_remove(inline_vec, i)
        }
    }

    /// Swap the Ts at the i'th and j'th indices in the vector v. Will abort if either of i or j are out of bounds
    /// for v.
    public fun swap<T: store>(v: &mut SmartVector<T>, i: u64, j: u64) {
        if (i > j) {
            return swap(v, j, i)
        };
        let len = length(v);
        assert!(j < len, error::invalid_argument(EINDEX_OUT_OF_BOUNDS));
        let inline_len = vector::length(&v.inline_vec);
        if (i >= inline_len) {
            big_vector::swap(option::borrow_mut(&mut v.big_vec), i - inline_len, j - inline_len);
        } else if (j < inline_len) {
            vector::swap(&mut v.inline_vec, i, j);
        } else {
            let big_vec = option::borrow_mut(&mut v.big_vec);
            let inline_vec = &mut v.inline_vec;
            let element_i = vector::swap_remove(inline_vec, i);
            let element_j = big_vector::swap_remove(big_vec, j - inline_len);
            vector::push_back(inline_vec, element_j);
            vector::swap(inline_vec, i, inline_len - 1);
            big_vector::push_back(big_vec, element_i);
            big_vector::swap(big_vec, j - inline_len, len - inline_len - 1);
        }
    }

    /// Reverse the order of the Ts in the vector v in-place.
    /// Disclaimer: This function may be costly. Use it at your own discretion.
    public fun reverse<T: store>(v: &mut SmartVector<T>) {
        let inline_len = vector::length(&v.inline_vec);
        let i = 0;
        let new_inline_vec = vector[];
        // Push the last `inline_len` Ts into a temp vector.
        while (i < inline_len) {
            vector::push_back(&mut new_inline_vec, pop_back(v));
            i = i + 1;
        };
        vector::reverse(&mut new_inline_vec);
        // Reverse the big_vector left if exists.
        if (option::is_some(&v.big_vec)) {
            big_vector::reverse(option::borrow_mut(&mut v.big_vec));
        };
        // Mem::swap the two vectors.
        let temp_vec = vector[];
        while (!vector::is_empty(&mut v.inline_vec)) {
            vector::push_back(&mut temp_vec, vector::pop_back(&mut v.inline_vec));
        };
        vector::reverse(&mut temp_vec);
        while (!vector::is_empty(&mut new_inline_vec)) {
            vector::push_back(&mut v.inline_vec, vector::pop_back(&mut new_inline_vec));
        };
        vector::destroy_empty(new_inline_vec);
        // Push the rest Ts originally left in inline_vector back to the end of the smart vector.
        while (!vector::is_empty(&mut temp_vec)) {
            push_back(v, vector::pop_back(&mut temp_vec));
        };
        vector::destroy_empty(temp_vec);
    }

    /// Return `(true, i)` if `val` is in the vector `v` at index `i`.
    /// Otherwise, returns `(false, 0)`.
    /// Disclaimer: This function may be costly. Use it at your own discretion.
    public fun index_of<T>(v: &SmartVector<T>, val: &T): (bool, u64) {
        let (found, i) = vector::index_of(&v.inline_vec, val);
        if (found) {
            (true, i)
        } else if (option::is_some(&v.big_vec)) {
            let (found, i) = big_vector::index_of(option::borrow(&v.big_vec), val);
            (found, i + vector::length(&v.inline_vec))
        } else {
            (false, 0)
        }
    }

    /// Return true if `val` is in the vector `v`.
    /// Disclaimer: This function may be costly. Use it at your own discretion.
    public fun contains<T>(v: &SmartVector<T>, val: &T): bool {
        if (is_empty(v)) return false;
        let (exist, _) = index_of(v, val);
        exist
    }

    /// Return the length of the vector.
    public fun length<T>(v: &SmartVector<T>): u64 {
        vector::length(&v.inline_vec) + if (option::is_none(&v.big_vec)) {
            0
        } else {
            big_vector::length(option::borrow(&v.big_vec))
        }
    }

    /// Return `true` if the vector `v` has no Ts and `false` otherwise.
    public fun is_empty<T>(v: &SmartVector<T>): bool {
        length(v) == 0
    }

    /// Apply the function to each T in the vector, consuming it.
    public inline fun for_each<T: store>(v: SmartVector<T>, f: |T|) {
        aptos_std::smart_vector::reverse(&mut v); // We need to reverse the vector to consume it efficiently
        aptos_std::smart_vector::for_each_reverse(v, |e| f(e));
    }

    /// Apply the function to each T in the vector, consuming it.
    public inline fun for_each_reverse<T>(v: SmartVector<T>, f: |T|) {
        let len = aptos_std::smart_vector::length(&v);
        while (len > 0) {
            f(aptos_std::smart_vector::pop_back(&mut v));
            len = len - 1;
        };
        aptos_std::smart_vector::destroy_empty(v)
    }

    /// Apply the function to a reference of each T in the vector.
    public inline fun for_each_ref<T>(v: &SmartVector<T>, f: |&T|) {
        let i = 0;
        let len = aptos_std::smart_vector::length(v);
        while (i < len) {
            f(aptos_std::smart_vector::borrow(v, i));
            i = i + 1
        }
    }

    /// Apply the function to a mutable reference to each T in the vector.
    public inline fun for_each_mut<T>(v: &mut SmartVector<T>, f: |&mut T|) {
        let i = 0;
        let len = aptos_std::smart_vector::length(v);
        while (i < len) {
            f(aptos_std::smart_vector::borrow_mut(v, i));
            i = i + 1
        }
    }

    /// Apply the function to a reference of each T in the vector with its index.
    public inline fun enumerate_ref<T>(v: &SmartVector<T>, f: |u64, &T|) {
        let i = 0;
        let len = aptos_std::smart_vector::length(v);
        while (i < len) {
            f(i, aptos_std::smart_vector::borrow(v, i));
            i = i + 1;
        };
    }

    /// Apply the function to a mutable reference of each T in the vector with its index.
    public inline fun enumerate_mut<T>(v: &mut SmartVector<T>, f: |u64, &mut T|) {
        let i = 0;
        let len = length(v);
        while (i < len) {
            f(i, borrow_mut(v, i));
            i = i + 1;
        };
    }

    /// Fold the function over the Ts. For example, `fold(vector[1,2,3], 0, f)` will execute
    /// `f(f(f(0, 1), 2), 3)`
    public inline fun fold<Accumulator, T: store>(
        v: SmartVector<T>,
        init: Accumulator,
        f: |Accumulator, T|Accumulator
    ): Accumulator {
        let accu = init;
        aptos_std::smart_vector::for_each(v, |elem| accu = f(accu, elem));
        accu
    }

    /// Fold right like fold above but working right to left. For example, `fold(vector[1,2,3], 0, f)` will execute
    /// `f(1, f(2, f(3, 0)))`
    public inline fun foldr<Accumulator, T>(
        v: SmartVector<T>,
        init: Accumulator,
        f: |T, Accumulator|Accumulator
    ): Accumulator {
        let accu = init;
        aptos_std::smart_vector::for_each_reverse(v, |elem| accu = f(elem, accu));
        accu
    }

    /// Map the function over the references of the Ts of the vector, producing a new vector without modifying the
    /// original vector.
    public inline fun map_ref<T1, T2: store>(
        v: &SmartVector<T1>,
        f: |&T1|T2
    ): SmartVector<T2> {
        let result = aptos_std::smart_vector::new<T2>();
        aptos_std::smart_vector::for_each_ref(v, |elem| aptos_std::smart_vector::push_back(&mut result, f(elem)));
        result
    }

    /// Map the function over the Ts of the vector, producing a new vector.
    public inline fun map<T1: store, T2: store>(
        v: SmartVector<T1>,
        f: |T1|T2
    ): SmartVector<T2> {
        let result = aptos_std::smart_vector::new<T2>();
        aptos_std::smart_vector::for_each(v, |elem| push_back(&mut result, f(elem)));
        result
    }

    /// Filter the vector using the boolean function, removing all Ts for which `p(e)` is not true.
    public inline fun filter<T: store + drop>(
        v: SmartVector<T>,
        p: |&T|bool
    ): SmartVector<T> {
        let result = aptos_std::smart_vector::new<T>();
        aptos_std::smart_vector::for_each(v, |elem| {
            if (p(&elem)) aptos_std::smart_vector::push_back(&mut result, elem);
        });
        result
    }

    public inline fun zip<T1: store, T2: store>(v1: SmartVector<T1>, v2: SmartVector<T2>, f: |T1, T2|) {
        // We need to reverse the vectors to consume it efficiently
        aptos_std::smart_vector::reverse(&mut v1);
        aptos_std::smart_vector::reverse(&mut v2);
        aptos_std::smart_vector::zip_reverse(v1, v2, |e1, e2| f(e1, e2));
    }

    /// Apply the function to each pair of elements in the two given vectors in the reverse order, consuming them.
    /// This errors out if the vectors are not of the same length.
    public inline fun zip_reverse<T1, T2>(
        v1: SmartVector<T1>,
        v2: SmartVector<T2>,
        f: |T1, T2|,
    ) {
        let len = aptos_std::smart_vector::length(&v1);
        // We can't use the constant ESMART_VECTORS_LENGTH_MISMATCH here as all calling code would then need to define it
        // due to how inline functions work.
        assert!(len == aptos_std::smart_vector::length(&v2), 0x20005);
        while (len > 0) {
            f(aptos_std::smart_vector::pop_back(&mut v1), aptos_std::smart_vector::pop_back(&mut v2));
            len = len - 1;
        };
        aptos_std::smart_vector::destroy_empty(v1);
        aptos_std::smart_vector::destroy_empty(v2);
    }

    /// Apply the function to the references of each pair of elements in the two given vectors.
    /// This errors out if the vectors are not of the same length.
    public inline fun zip_ref<T1, T2>(
        v1: &SmartVector<T1>,
        v2: &SmartVector<T2>,
        f: |&T1, &T2|,
    ) {
        let len = aptos_std::smart_vector::length(v1);
        // We can't use the constant ESMART_VECTORS_LENGTH_MISMATCH here as all calling code would then need to define it
        // due to how inline functions work.
        assert!(len == aptos_std::smart_vector::length(v2), 0x20005);
        let i = 0;
        while (i < len) {
            f(aptos_std::smart_vector::borrow(v1, i), aptos_std::smart_vector::borrow(v2, i));
            i = i + 1
        }
    }

    /// Apply the function to mutable references to each pair of elements in the two given vectors.
    /// This errors out if the vectors are not of the same length.
    public inline fun zip_mut<T1, T2>(
        v1: &mut SmartVector<T1>,
        v2: &mut SmartVector<T2>,
        f: |&mut T1, &mut T2|,
    ) {
        let i = 0;
        let len = aptos_std::smart_vector::length(v1);
        // We can't use the constant ESMART_VECTORS_LENGTH_MISMATCH here as all calling code would then need to define it
        // due to how inline functions work.
        assert!(len == aptos_std::smart_vector::length(v2), 0x20005);
        while (i < len) {
            f(aptos_std::smart_vector::borrow_mut(v1, i), aptos_std::smart_vector::borrow_mut(v2, i));
            i = i + 1
        }
    }

    /// Map the function over the element pairs of the two vectors, producing a new vector.
    public inline fun zip_map<T1: store, T2: store, NewT: store>(
        v1: SmartVector<T1>,
        v2: SmartVector<T2>,
        f: |T1, T2|NewT
    ): SmartVector<NewT> {
        // We can't use the constant ESMART_VECTORS_LENGTH_MISMATCH here as all calling code would then need to define it
        // due to how inline functions work.
        assert!(aptos_std::smart_vector::length(&v1) == aptos_std::smart_vector::length(&v2), 0x20005);

        let result = aptos_std::smart_vector::new<NewT>();
        aptos_std::smart_vector::zip(v1, v2, |e1, e2| push_back(&mut result, f(e1, e2)));
        result
    }

    /// Map the function over the references of the element pairs of two vectors, producing a new vector from the return
    /// values without modifying the original vectors.
    public inline fun zip_map_ref<T1, T2, NewT: store>(
        v1: &SmartVector<T1>,
        v2: &SmartVector<T2>,
        f: |&T1, &T2|NewT
    ): SmartVector<NewT> {
        // We can't use the constant ESMART_VECTORS_LENGTH_MISMATCH here as all calling code would then need to define it
        // due to how inline functions work.
        assert!(aptos_std::smart_vector::length(v1) == aptos_std::smart_vector::length(v2), 0x20005);

        let result = aptos_std::smart_vector::new<NewT>();
        aptos_std::smart_vector::zip_ref(v1, v2, |e1, e2| push_back(&mut result, f(e1, e2)));
        result
    }

    #[test]
    fun smart_vector_test() {
        let v = empty();
        let i = 0;
        while (i < 100) {
            push_back(&mut v, i);
            i = i + 1;
        };
        let j = 0;
        while (j < 100) {
            let val = borrow(&v, j);
            assert!(*val == j, 0);
            j = j + 1;
        };
        while (i > 0) {
            i = i - 1;
            let (exist, index) = index_of(&v, &i);
            let j = pop_back(&mut v);
            assert!(exist, 0);
            assert!(index == i, 0);
            assert!(j == i, 0);
        };
        while (i < 100) {
            push_back(&mut v, i);
            i = i + 1;
        };
        let last_index = length(&v) - 1;
        assert!(swap_remove(&mut v, last_index) == 99, 0);
        assert!(swap_remove(&mut v, 0) == 0, 0);
        while (length(&v) > 0) {
            // the vector is always [N, 1, 2, ... N-1] with repetitive swap_remove(&mut v, 0)
            let expected = length(&v);
            let val = swap_remove(&mut v, 0);
            assert!(val == expected, 0);
        };
        destroy_empty(v);
    }

    #[test]
    fun smart_vector_append_edge_case_test() {
        let v1 = empty();
        let v2 = singleton(1u64);
        let v3 = empty();
        let v4 = empty();
        append(&mut v3, v4);
        assert!(length(&v3) == 0, 0);
        append(&mut v2, v3);
        assert!(length(&v2) == 1, 0);
        append(&mut v1, v2);
        assert!(length(&v1) == 1, 0);
        destroy(v1);
    }

    #[test]
    fun smart_vector_append_test() {
        let v1 = empty();
        let v2 = empty();
        let i = 0;
        while (i < 7) {
            push_back(&mut v1, i);
            i = i + 1;
        };
        while (i < 25) {
            push_back(&mut v2, i);
            i = i + 1;
        };
        append(&mut v1, v2);
        assert!(length(&v1) == 25, 0);
        i = 0;
        while (i < 25) {
            assert!(*borrow(&v1, i) == i, 0);
            i = i + 1;
        };
        destroy(v1);
    }

    #[test]
    fun smart_vector_remove_test() {
        let v = empty();
        let i = 0u64;
        while (i < 101) {
            push_back(&mut v, i);
            i = i + 1;
        };
        let inline_len = vector::length(&v.inline_vec);
        remove(&mut v, 100);
        remove(&mut v, 90);
        remove(&mut v, 80);
        remove(&mut v, 70);
        remove(&mut v, 60);
        remove(&mut v, 50);
        remove(&mut v, 40);
        remove(&mut v, 30);
        remove(&mut v, 20);
        assert!(vector::length(&v.inline_vec) == inline_len, 0);
        remove(&mut v, 10);
        assert!(vector::length(&v.inline_vec) + 1 == inline_len, 0);
        remove(&mut v, 0);
        assert!(vector::length(&v.inline_vec) + 2 == inline_len, 0);
        assert!(length(&v) == 90, 0);

        let index = 0;
        i = 0;
        while (i < 101) {
            if (i % 10 != 0) {
                assert!(*borrow(&v, index) == i, 0);
                index = index + 1;
            };
            i = i + 1;
        };
        destroy(v);
    }

    #[test]
    fun smart_vector_reverse_test() {
        let v = empty();
        let i = 0u64;
        while (i < 10) {
            push_back(&mut v, i);
            i = i + 1;
        };
        reverse(&mut v);
        let k = 0;
        while (k < 10) {
            assert!(*vector::borrow(&v.inline_vec, k) == 9 - k, 0);
            k = k + 1;
        };
        while (i < 100) {
            push_back(&mut v, i);
            i = i + 1;
        };
        while (!vector::is_empty(&v.inline_vec)) {
            remove(&mut v, 0);
        };
        reverse(&mut v);
        i = 0;
        let len = length(&v);
        while (i + 1 < len) {
            assert!(
                *big_vector::borrow(option::borrow(&v.big_vec), i) == *big_vector::borrow(
                    option::borrow(&v.big_vec),
                    i + 1
                ) + 1,
                0
            );
            i = i + 1;
        };
        destroy(v);
    }

    #[test]
    fun smart_vector_add_all_test() {
        let v = empty_with_config(1, 2);
        add_all(&mut v, vector[1, 2, 3, 4, 5, 6]);
        assert!(length(&v) == 6, 0);
        let i = 0;
        while (i < 6) {
            assert!(*borrow(&v, i) == i + 1, 0);
            i = i + 1;
        };
        destroy(v);
    }

    #[test]
    fun smart_vector_to_vector_test() {
        let v1 = empty_with_config(7, 11);
        let i = 0;
        while (i < 100) {
            push_back(&mut v1, i);
            i = i + 1;
        };
        let v2 = to_vector(&v1);
        let j = 0;
        while (j < 100) {
            assert!(*vector::borrow(&v2, j) == j, 0);
            j = j + 1;
        };
        destroy(v1);
    }

    #[test]
    fun smart_vector_swap_test() {
        let v = empty();
        let i = 0;
        while (i < 101) {
            push_back(&mut v, i);
            i = i + 1;
        };
        i = 0;
        while (i < 51) {
            swap(&mut v, i, 100 - i);
            i = i + 1;
        };
        i = 0;
        while (i < 101) {
            assert!(*borrow(&v, i) == 100 - i, 0);
            i = i + 1;
        };
        destroy(v);
    }

    #[test]
    fun smart_vector_index_of_test() {
        let v = empty();
        let i = 0;
        while (i < 100) {
            push_back(&mut v, i);
            let (found, idx) = index_of(&mut v, &i);
            assert!(found && idx == i, 0);
            i = i + 1;
        };
        destroy(v);
    }
}
