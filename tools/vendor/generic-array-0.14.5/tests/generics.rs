#![recursion_limit = "128"]

#[macro_use]
extern crate generic_array;

use generic_array::typenum::consts::U4;

use std::fmt::Debug;
use std::ops::Add;

use generic_array::{GenericArray, ArrayLength};
use generic_array::sequence::*;
use generic_array::functional::*;

/// Example function using generics to pass N-length sequences and map them
pub fn generic_map<S>(s: S)
where
    S: FunctionalSequence<i32>,            // `.map`
    S::Item: Add<i32, Output = i32>,       // `x + 1`
    S: MappedGenericSequence<i32, i32>,    // `i32` -> `i32`
    MappedSequence<S, i32, i32>: Debug,    // println!
{
    let a = s.map(|x| x + 1);

    println!("{:?}", a);
}

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

/// Super-simple fixed-length i32 `GenericArray`s
pub fn generic_array_plain_zip_sum(a: GenericArray<i32, U4>, b: GenericArray<i32, U4>) -> i32 {
    a.zip(b, |l, r| l + r).map(|x| x + 1).fold(0, |a, x| x + a)
}

pub fn generic_array_variable_length_zip_sum<N>(a: GenericArray<i32, N>, b: GenericArray<i32, N>) -> i32
where
    N: ArrayLength<i32>,
{
    a.zip(b, |l, r| l + r).map(|x| x + 1).fold(0, |a, x| x + a)
}

pub fn generic_array_same_type_variable_length_zip_sum<T, N>(a: GenericArray<T, N>, b: GenericArray<T, N>) -> i32
where
    N: ArrayLength<T> + ArrayLength<<T as Add<T>>::Output>,
    T: Add<T, Output=i32>,
{
    a.zip(b, |l, r| l + r).map(|x| x + 1).fold(0, |a, x| x + a)
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
    a.zip(b, |l, r| l + r).map(|x| x + 1).fold(0, |a, x| x + a)
}

#[test]
fn test_generics() {
    generic_map(arr![i32; 1, 2, 3, 4]);

    assert_eq!(generic_sequence_zip_sum(arr![i32; 1, 2, 3, 4], arr![i32; 2, 3, 4, 5]), 28);

    assert_eq!(generic_array_plain_zip_sum(arr![i32; 1, 2, 3, 4], arr![i32; 2, 3, 4, 5]), 28);

    assert_eq!(generic_array_variable_length_zip_sum(arr![i32; 1, 2, 3, 4], arr![i32; 2, 3, 4, 5]), 28);

    assert_eq!(generic_array_same_type_variable_length_zip_sum(arr![i32; 1, 2, 3, 4], arr![i32; 2, 3, 4, 5]), 28);

    assert_eq!(generic_array_zip_sum(arr![i32; 1, 2, 3, 4], arr![i32; 2, 3, 4, 5]), 28);
}