// Copyright 2017 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

// Indicates that the element is not encoded; there is no *R* factor
// that needs to be canceled out.
#[derive(Copy, Clone)]
pub enum Unencoded {}

// Indicates that the element is encoded; the value has one *R*
// factor that needs to be canceled out.
#[derive(Copy, Clone)]
pub enum R {}

// Indicates the element is encoded twice; the value has two *R*
// factors that need to be canceled out.
#[derive(Copy, Clone)]
pub enum RR {}

// Indicates the element is inversely encoded; the value has one
// 1/*R* factor that needs to be canceled out.
#[derive(Copy, Clone)]
pub enum RInverse {}

pub trait Encoding {}

impl Encoding for RR {}
impl Encoding for R {}
impl Encoding for Unencoded {}
impl Encoding for RInverse {}

/// The encoding of the result of a reduction.
pub trait ReductionEncoding {
    type Output: Encoding;
}

impl ReductionEncoding for RR {
    type Output = R;
}
impl ReductionEncoding for R {
    type Output = Unencoded;
}
impl ReductionEncoding for Unencoded {
    type Output = RInverse;
}

/// The encoding of the result of a multiplication.
pub trait ProductEncoding {
    type Output: Encoding;
}

impl<E: ReductionEncoding> ProductEncoding for (Unencoded, E) {
    type Output = E::Output;
}

impl<E: Encoding> ProductEncoding for (R, E) {
    type Output = E;
}

impl<E: ReductionEncoding> ProductEncoding for (RInverse, E)
where
    E::Output: ReductionEncoding,
{
    type Output = <<E as ReductionEncoding>::Output as ReductionEncoding>::Output;
}

// XXX: Rust doesn't allow overlapping impls,
// TODO (if/when Rust allows it):
// impl<E1, E2: ReductionEncoding> ProductEncoding for
//         (E1, E2) {
//     type Output = <(E2, E1) as ProductEncoding>::Output;
// }
impl ProductEncoding for (RR, Unencoded) {
    type Output = <(Unencoded, RR) as ProductEncoding>::Output;
}
impl ProductEncoding for (RR, RInverse) {
    type Output = <(RInverse, RR) as ProductEncoding>::Output;
}
