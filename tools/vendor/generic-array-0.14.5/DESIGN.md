Design and Usage Notes
======================

## Sections

1. [How it Works](#how-it-works)
2. [Initialization](#initialization)
3. [Functional Programming](#functional-programming)
4. [Miscellaneous Utilities](#miscellaneous-utilities)
5. [Safety](#safety)
6. [Optimization](#optimization)
7. [The Future](#the-future)

**NOTE**: This document uses `<details>` sections, so look out for collapsible parts with an arrow on the left.

# How it works

`generic-array` is a method of achieving fixed-length fixed-size stack-allocated generic arrays without needing const generics in stable Rust.

That is to say this:

```rust
struct Foo<const N: usize> {
    data: [i32; N],
}
```

or anything similar is not currently supported.

However, Rust's type system is sufficiently advanced, and a "hack" for solving this was created in the form of the `typenum` crate, which recursively defines integer values in binary as nested types, and operations which can be applied to those type-numbers, such as `Add`, `Sub`, etc.

e.g. `6` would be `UInt<UInt<UInt<UTerm, B1>, B1>, B0>`

Over time, I've come to see `typenum` as less of a hack and more as an elegant solution.

The recursive binary nature of `typenum` is what makes `generic-array` possible, so:

```rust
struct Foo<N: ArrayLength<i32>> {
    data: GenericArray<i32, N>,
}
```

is supported.

I often see questions about why `ArrayLength` requires the element type `T` in it's signature, even though it's not used in the inner `ArrayType`.

This is because `GenericArray` itself does not define the actual array. Rather, it is defined as:

```rust
pub struct GenericArray<T, N: ArrayLength<T>> {
    data: N::ArrayType,
}
```

The trait `ArrayLength` does all the real heavy lifting for defining the data, with implementations on `UInt<N, B0>`, `UInt<N, B1>` and `UTerm`, which correspond to even, odd and zero numeric values, respectively.

`ArrayLength`'s implementations use type-level recursion to peel away each least significant bit and form sort of an opaque binary tree of contiguous data the correct physical size to store `N` elements of `T`. The tree, or block of data, is then stored inside of `GenericArray` to be reinterpreted as the array.

For example, `GenericArray<T, U6>` more or less expands to (at compile time):

<details>
<summary>Expand for code</summary>

```rust
GenericArray {
    // UInt<UInt<UInt<UTerm, B1>, B1>, B0>
    data: EvenData {
        // UInt<UInt<UTerm, B1>, B1>
        left: OddData {
            // UInt<UTerm, B1>
            left: OddData {
                left: (),  // UTerm
                right: (), // UTerm
                data: T,   // Element 0
            },
            // UInt<UTerm, B1>
            right: OddData {
                left: (),  // UTerm
                right: (), // UTerm
                data: T,   // Element 1
            },
            data: T        // Element 2
        },
        // UInt<UInt<UTerm, B1>, B1>
        right: OddData {
            // UInt<UTerm, B1>
            left: OddData {
                left: (),  // UTerm
                right: (), // UTerm
                data: T,   // Element 3
            },
            // UInt<UTerm, B1>
            right: OddData {
                left: (),  // UTerm
                right: (), // UTerm
                data: T,   // Element 4
            },
            data: T        // Element 5
        }
    }
}
```

</details>

This has the added benefit of only being `log2(N)` deep, which is important for things like `Drop`, which we'll go into later.

Then, we take `data` and cast it to `*const T` or `*mut T` and use it as a slice like:

```rust
unsafe {
    slice::from_raw_parts(
        self as *const Self as *const T,
        N::to_usize()
    )
}
```

It is useful to note that because `typenum` is compile-time with nested generics, `to_usize`, even if it isn't a `const fn`, *does* expand to effectively `1 + 2 + 4 + 8 + ...` and so forth, which LLVM is smart enough to reduce to a single compile-time constant. This helps hint to the optimizers about things such as bounds checks.

So, to reiterate, we're working with a raw block of contiguous memory the correct physical size to store `N` elements of `T`. It's really no different from how normal arrays are stored.

## Pointer Safety

Of course, casting pointers around and constructing blocks of data out of thin air is normal for C, but here in Rust we try to be a bit less prone to segfaults. Therefore, great care is taken to minimize casual `unsafe` usage and restrict `unsafe` to specific parts of the API, making heavy use those exposed safe APIs internally.

For example, the above `slice::from_raw_parts` is only used twice in the entire library, once for `&[T]` and `slice::from_raw_parts_mut` once for `&mut [T]`. Everything else goes through those slices.

# Initialization

## Constant

"Constant" initialization, that is to say - without dynamic values, can be done via the `arr![]` macro, which works almost exactly like `vec![]`, but with an additional type parameter.

Example:

```rust
let my_arr = arr![i32; 1, 2, 3, 4, 5, 6, 7, 8];
```

## Dynamic

Although some users have opted to use their own initializers, as of version `0.9` and beyond `generic-array` includes safe methods for initializing elements in the array.

The `GenericSequence` trait defines a `generate` method which can be used like so:

```rust
use generic_array::{GenericArray, sequence::GenericSequence};

let squares: GenericArray<i32, U4> =
             GenericArray::generate(|i: usize| i as i32 * 2);
```

and `GenericArray` additionally implements `FromIterator`, although `from_iter` ***will*** panic if the number of elements is not *at least* `N`. It will ignore extra items.

The safety of these operations is described later.

# Functional Programming

In addition to `GenericSequence`, this crate provides a `FunctionalSequence`, which allows extremely efficient `map`, `zip` and `fold` operations on `GenericArray`s.

As described at the end of the [Optimization](#optimization) section, `FunctionalSequence` uses clever specialization tactics to provide optimized methods wherever possible, while remaining perfectly safe.

Some examples, taken from `tests/generic.rs`:

<details>
<summary>Expand for code</summary>

This is so extensive to show how you can build up to processing totally arbitrary sequences, but for the most part these can be used on `GenericArray` instances without much added complexity.

```rust
/// Super-simple fixed-length i32 `GenericArray`s
pub fn generic_array_plain_zip_sum(a: GenericArray<i32, U4>, b: GenericArray<i32, U4>) -> i32 {
    a.zip(b, |l, r| l + r)
     .map(|x| x + 1)
     .fold(0, |a, x| x + a)
}

pub fn generic_array_variable_length_zip_sum<N>(a: GenericArray<i32, N>, b: GenericArray<i32, N>) -> i32
where
    N: ArrayLength<i32>,
{
    a.zip(b, |l, r| l + r)
     .map(|x| x + 1)
     .fold(0, |a, x| x + a)
}

pub fn generic_array_same_type_variable_length_zip_sum<T, N>(a: GenericArray<T, N>, b: GenericArray<T, N>) -> i32
where
    N: ArrayLength<T> + ArrayLength<<T as Add<T>>::Output>,
    T: Add<T, Output=i32>,
{
    a.zip(b, |l, r| l + r)
     .map(|x| x + 1)
     .fold(0, |a, x| x + a)
}

/// Complex example using fully generic `GenericArray`s with the same length.
///
/// It's mostly just the repeated `Add` traits, which would be present in other systems anyway.
pub fn generic_array_zip_sum<A, B, N: ArrayLength<A> + ArrayLength<B>>(a: GenericArray<A, N>, b: GenericArray<B, N>) -> i32
where
    A: Add<B>,
    N: ArrayLength<<A as Add<B>>::Output> +
        ArrayLength<<<A as Add<B>>::Output as Add<i32>>::Output>,
    <A as Add<B>>::Output: Add<i32>,
    <<A as Add<B>>::Output as Add<i32>>::Output: Add<i32, Output=i32>,
{
    a.zip(b, |l, r| l + r)
     .map(|x| x + 1)
     .fold(0, |a, x| x + a)
}
```
</details>

and if you really want to go off the deep end and support any arbitrary *`GenericSequence`*:

<details>
<summary>Expand for code</summary>

```rust
/// Complex example function using generics to pass N-length sequences, zip them, and then map that result.
///
/// If used with `GenericArray` specifically this isn't necessary
pub fn generic_sequence_zip_sum<A, B>(a: A, b: B) -> i32
where
    A: FunctionalSequence<i32>,                                                                 // `.zip`
    B: FunctionalSequence<i32, Length = A::Length>,                                             // `.zip`
    A: MappedGenericSequence<i32, i32>,                                                         // `i32` -> `i32`
    B: MappedGenericSequence<i32, i32, Mapped = MappedSequence<A, i32, i32>>,                   // `i32` -> `i32`, prove A and B can map to the same output
    A::Item: Add<B::Item, Output = i32>,                                                        // `l + r`
    MappedSequence<A, i32, i32>: MappedGenericSequence<i32, i32> + FunctionalSequence<i32>,     // `.map`
    SequenceItem<MappedSequence<A, i32, i32>>: Add<i32, Output=i32>,                            // `x + 1`
    MappedSequence<MappedSequence<A, i32, i32>, i32, i32>: Debug,                               // `println!`
    MappedSequence<MappedSequence<A, i32, i32>, i32, i32>: FunctionalSequence<i32>,             // `.fold`
    SequenceItem<MappedSequence<MappedSequence<A, i32, i32>, i32, i32>>: Add<i32, Output=i32>   // `x + a`, note the order
{
    let c = a.zip(b, |l, r| l + r).map(|x| x + 1);

    println!("{:?}", c);

    c.fold(0, |a, x| x + a)
}
```

of course, as I stated before, that's almost never necessary, especially when you know the concrete types of all the components.

</details>

The [`numeric-array`](https://crates.io/crates/numeric-array) crate uses these to apply numeric operations across all elements in a `GenericArray`, making full use of all the optimizations described in the last section here.

# Miscellaneous Utilities

Although not usually advertised, `generic-array` contains traits for lengthening, shortening, splitting and concatenating arrays.

For example, these snippets are taken from `tests/mod.rs`:

<details>
<summary>Expand for code</summary>

Appending and prepending elements:

```rust
use generic_array::sequence::Lengthen;

#[test]
fn test_append() {
    let a = arr![i32; 1, 2, 3];

    let b = a.append(4);

    assert_eq!(b, arr![i32; 1, 2, 3, 4]);
}

#[test]
fn test_prepend() {
    let a = arr![i32; 1, 2, 3];

    let b = a.prepend(4);

    assert_eq!(b, arr![i32; 4, 1, 2, 3]);
}
```

Popping elements from the front of back of the array:

```rust
use generic_array::sequence::Shorten;

let a = arr![i32; 1, 2, 3, 4];

let (init, last) = a.pop_back();

assert_eq!(init, arr![i32; 1, 2, 3]);
assert_eq!(last, 4);

let (head, tail) = a.pop_front();

assert_eq!(head, 1);
assert_eq!(tail, arr![i32; 2, 3, 4]);
```

and of course concatenating and splitting:

```rust
use generic_array::sequence::{Concat, Split};

let a = arr![i32; 1, 2];
let b = arr![i32; 3, 4];

let c = a.concat(b);

assert_eq!(c, arr![i32; 1, 2, 3, 4]);

let (d, e) = c.split();

assert_eq!(d, arr![i32; 1]);
assert_eq!(e, arr![i32; 2, 3, 4]);
```
</details>

`Split` and `Concat` in these examples use type-inference to determine the lengths of the resulting arrays.

# Safety

As stated earlier, for raw reinterpretations such as this, safety is a must even while working with unsafe code. Great care is taken to reduce or eliminate undefined behavior.

For most of the above code examples, the biggest potential undefined behavior hasn't even been applicable for one simple reason: they were all primitive values.

The simplest way to lead into this is to post these questions:

1. What if the element type of the array implements `Drop`?
2. What if `GenericArray::generate` opens a bunch of files?
3. What if halfway through opening each of the files, one is not found?
4. What if the resulting error is unwrapped, causing the generation function to panic?

For a fully initialized `GenericArray`, the expanded structure as described in the [How It Works](#how-it-works) can implement `Drop` naturally, recursively dropping elements. As it is only `log2(N)` deep, the recursion is very small overall.

In fact, I tested it while writing this, the size of the array itself overflows the stack before any recursive calls to `drop` can.

However, ***partially*** initialized arrays, such as described in the above hypothetical, pose an issue where `drop` could be called on uninitialized data, which is undefined behavior.

To solve this, `GenericArray` implements two components named `ArrayBuilder` and `ArrayConsumer`, which work very similarly.

`ArrayBuilder` creates a block of wholly uninitialized memory via `mem::unintialized()`, and stores that in a `ManuallyDrop` wrapper. `ManuallyDrop` does exactly what it says on the tin, and simply doesn't drop the value unless manually requested to.

So, as we're initializing our array, `ArrayBuilder` keeps track of the current position through it, and if something happens, `ArrayBuilder` itself will iteratively and manually `drop` all currently initialized elements, ignoring any uninitialized ones, because those are just raw memory and should be ignored.

`ArrayConsumer` does almost the same, "moving" values out of the array and into something else, like user code. It uses `ptr::read` to "move" the value out, and increments a counter saying that value is no longer valid in the array.

If a panic occurs in the user code with that element, it's dropped naturally as it was moved into that scope. `ArrayConsumer` then proceeds to iteratively and manually `drop` all *remaining* elements.

Combined, these two systems provide a safe system for building and consuming `GenericArray`s. In fact, they are used extensively inside the library itself for `FromIterator`, `GenericSequence` and `FunctionalSequence`, among others.

Even `GenericArray`s implementation of `Clone` makes use of this via:

```rust
impl<T: Clone, N> Clone for GenericArray<T, N>
where
    N: ArrayLength<T>,
{
    fn clone(&self) -> GenericArray<T, N> {
        self.map(|x| x.clone())
    }
}
```

where `.map` is from the `FunctionalSequence`, and uses those builder and consumer structures to safely move and initialize values. Although, in this particular case, a consumer is not necessary as we're using references. More on how that is automatically deduced is described in the next section.

# Optimization

Rust and LLVM is smart. Crazy smart. However, it's not magic.

In my experience, most of Rust's "zero-cost" abstractions stem more from the type system, rather than explicit optimizations. Most Rust code is very easily optimizable and inlinable by design, so it can be simplified and compacted rather well, as opposed to the spaghetti code of some other languages.

Unfortunately, unless `rustc` or LLVM can "prove" things about code to simplify it, it must still be run, and can prevent further optimization.

A great example of this, and why I created the `GenericSequence` and `FunctionalSequence` traits, are iterators.

Custom iterators are slow. Not terribly slow, but slow enough to prevent some rather important optimizations.

Take `GenericArrayIter` for example:

<details>
<summary>Expand for code</summary>

```rust
pub struct GenericArrayIter<T, N: ArrayLength<T>> {
    array: ManuallyDrop<GenericArray<T, N>>,
    index: usize,
    index_back: usize,
}

impl<T, N> Iterator for GenericArrayIter<T, N>
where
    N: ArrayLength<T>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.index < self.index_back {
            let p = unsafe {
                Some(ptr::read(self.array.get_unchecked(self.index)))
            };

            self.index += 1;

            p
        } else {
            None
        }
    }

    //and more
}
```
</details>

Seems simple enough, right? Move an element out of the array with `ptr::read` and increment the index. If the iterator is dropped, the remaining elements are dropped exactly as they would with `ArrayConsumer`. `index_back` is provided for `DoubleEndedIterator`.

Unfortunately, that single `if` statement is terrible. In my mind, this is one of the biggest flaws of the iterator design. A conditional jump on a mutable variable unrelated to the data we are accessing on each call foils the optimizer and generates suboptimal code for the above iterator, even when we use `get_unchecked`.

The optimizer is unable to see that we are simply accessing memory sequentially. In fact, almost all iterators are like this. Granted, this is usually fine and, especially if they have to handle errors, it's perfectly acceptable.

However, there is one iterator in the standard library that is optimized perfectly: the slice iterator. So perfectly in fact that it allows the optimizer to do something even more special: **auto-vectorization**! We'll get to that later.

It's a bit frustrating as to *why* slice iterators can be so perfectly optimized, and it basically boils down to that the iterator itself does not own the data the slice refers to, so it uses raw pointers to the array/sequence/etc. rather than having to use an index on a stack allocated and always moving array. It can check for if the iterator is empty by comparing some `front` and `back` pointers for equality, and because those directly correspond to the position in memory of the next element, LLVM can see that and make optimizations.

So, the gist of that is: always use slice iterators where possible.

Here comes the most important part of all of this: `ArrayBuilder` and `ArrayConsumer` don't iterate the arrays themselves. Instead, we use slice iterators (immutable and mutable), with `zip` or `enumerate`, to apply operations to the entire array, incrementing the position in both `ArrayBuilder` or `ArrayConsumer` to keep track.

For example, `GenericSequence::generate` for `GenericArray` is:

<details>
<summary>Expand for code</summary>

```rust
fn generate<F>(mut f: F) -> GenericArray<T, N>
where
    F: FnMut(usize) -> T,
{
    unsafe {
        let mut destination = ArrayBuilder::new();

        {
            let (destination_iter, position) = destination.iter_position();

            for (i, dst) in destination_iter.enumerate() {
                ptr::write(dst, f(i));

                *position += 1;
            }
        }

        destination.into_inner()
    }
}
```

where `ArrayBuilder::iter_position` is just an internal convenience function:

```rust
pub unsafe fn iter_position(&mut self) -> (slice::IterMut<T>, &mut usize) {
    (self.array.iter_mut(), &mut self.position)
}
```
</details>

Of course, this may appear to be redundant, if we're using an iterator that keeps track of the position itself, and the builder is also keeping track of the position. However, the two are decoupled.

If the generation function doesn't have a chance at panicking, and/or the array element type doesn't implement `Drop`, the optimizer deems the `Drop` implementation on `ArrayBuilder` (and `ArrayConsumer`) dead code, and therefore `position` is never actually read from, so it becomes dead code as well, and is removed.

So for simple non-`Drop`/non-panicking elements and generation functions, `generate` becomes a very simple loop that uses a slice iterator to write values to the array.

Next, let's take a look at a more complex example where this *really* shines: `.zip`

To cut down on excessively verbose code, `.zip` uses `FromIterator` for building the array, which has almost identical code to `generate`, so it will be omitted.

The first implementation of `.zip` is defined as:

<details>
<summary>Expand for code</summary>

```rust
fn inverted_zip<B, U, F>(
    self,
    lhs: GenericArray<B, Self::Length>,
    mut f: F,
) -> MappedSequence<GenericArray<B, Self::Length>, B, U>
where
    GenericArray<B, Self::Length>:
        GenericSequence<B, Length = Self::Length> + MappedGenericSequence<B, U>,
    Self: MappedGenericSequence<T, U>,
    Self::Length: ArrayLength<B> + ArrayLength<U>,
    F: FnMut(B, Self::Item) -> U,
{
    unsafe {
        let mut left = ArrayConsumer::new(lhs);
        let mut right = ArrayConsumer::new(self);

        let (left_array_iter, left_position) = left.iter_position();
        let (right_array_iter, right_position) = right.iter_position();

        FromIterator::from_iter(left_array_iter.zip(right_array_iter).map(|(l, r)| {
            let left_value = ptr::read(l);
            let right_value = ptr::read(r);

            *left_position += 1;
            *right_position += 1;

            f(left_value, right_value)
        }))
    }
}
```
</details>

The gist of this is that we have two `GenericArray` instances that need to be zipped together and mapped to a new sequence. This employs two `ArrayConsumer`s, and more or less use the same pattern as the previous example.

Again, the position values can be optimized out, and so can the slice iterator adapters.

We can go a step further with this, however.

Consider this:

```rust
let a = arr![i32; 1, 3, 5, 7];
let b = arr![i32; 2, 4, 6, 8];

let c = a.zip(b, |l, r| l + r);

assert_eq!(c, arr![i32; 3, 7, 11, 15]);
```

when compiled with:

```
cargo rustc --lib --profile test --release -- -C target-cpu=native -C opt-level=3 --emit asm
```

will produce assembly with the following relevant instructions taken from the entire program:

```asm
; Copy constant to register
vmovaps  __xmm@00000007000000050000000300000001(%rip), %xmm0

; Copy constant to register
vmovaps  __xmm@00000008000000060000000400000002(%rip), %xmm0

; Add the two values together
vpaddd   192(%rsp), %xmm0, %xmm1

; Copy constant to register
vmovaps  __xmm@0000000f0000000b0000000700000003(%rip), %xmm0

; Compare result of the addition with the last constant
vpcmpeqb 128(%rsp), %xmm0, %xmm0
```

so, aside from a bunch of obvious hygiene instructions around those selected instructions,
it seriously boils down that `.zip` call to a ***SINGLE*** SIMD instruction. In fact, it continues to do this for even larger arrays. Although it does fall back to individual additions for fewer than four elements, as it can't fit those into an SSE register evenly.

Using this property of auto-vectorization without sacrificing safety, I created the [`numeric-array`](https://crates.io/crates/numeric-array) crate which makes use of this to wrap `GenericArray` and implement numeric traits so that almost *all* operations can be auto-vectorized, even complex ones like fused multiple-add.

It doesn't end there, though. You may have noticed that the function name for zip above wasn't `zip`, but `inverted_zip`.

This is because `generic-array` employs a clever specialization tactic to ensure `.zip` works corrects with:

1. `a.zip(b, ...)`
2. `(&a).zip(b, ...)`
3. `(&a).zip(&b, ...)`
4. `a.zip(&b, ...)`

wherein `GenericSequence` and `FunctionalSequence` have default implementations of `zip` variants, with concrete implementations for `GenericArray`. As `GenericSequence` is implemented for `&GenericArray`, where calling `into_iter` on produces a slice iterator, it can use "naive" iterator adapters to the same effect, while the specialized implementations use `ArrayConsumer`.

The result is that any combination of move or reference calls to `.zip`, `.map` and `.fold` produce code that can be optimized, none of them falling back to slow non-slice iterators. All perfectly safe with the `ArrayBuilder` and `ArrayConsumer` systems.

Honestly, `GenericArray` is better than standard arrays at this point.

# The Future

If/when const generics land in stable Rust, my intention is to reorient this crate or create a new crate to provide traits and wrappers for standard arrays to provide the same safety and performance discussed above.