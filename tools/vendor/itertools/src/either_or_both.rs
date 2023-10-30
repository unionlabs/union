use crate::EitherOrBoth::*;

use either::Either;

/// Value that either holds a single A or B, or both.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EitherOrBoth<A, B> {
    /// Both values are present.
    Both(A, B),
    /// Only the left value of type `A` is present.
    Left(A),
    /// Only the right value of type `B` is present.
    Right(B),
}

impl<A, B> EitherOrBoth<A, B> {
    /// If `Left`, or `Both`, return true, otherwise, return false.
    pub fn has_left(&self) -> bool {
        self.as_ref().left().is_some()
    }

    /// If `Right`, or `Both`, return true, otherwise, return false.
    pub fn has_right(&self) -> bool {
        self.as_ref().right().is_some()
    }

    /// If Left, return true otherwise, return false.
    /// Exclusive version of [`has_left`](EitherOrBoth::has_left).
    pub fn is_left(&self) -> bool {
        match *self {
            Left(_) => true,
            _ => false,
        }
    }

    /// If Right, return true otherwise, return false.
    /// Exclusive version of [`has_right`](EitherOrBoth::has_right).
    pub fn is_right(&self) -> bool {
        match *self {
            Right(_) => true,
            _ => false,
        }
    }

    /// If Right, return true otherwise, return false.
    /// Equivalent to `self.as_ref().both().is_some()`.
    pub fn is_both(&self) -> bool {
        self.as_ref().both().is_some()
    }

    /// If `Left`, or `Both`, return `Some` with the left value, otherwise, return `None`.
    pub fn left(self) -> Option<A> {
        match self {
            Left(left) | Both(left, _) => Some(left),
            _ => None,
        }
    }

    /// If `Right`, or `Both`, return `Some` with the right value, otherwise, return `None`.
    pub fn right(self) -> Option<B> {
        match self {
            Right(right) | Both(_, right) => Some(right),
            _ => None,
        }
    }

    /// If Both, return `Some` tuple containing left and right.
    pub fn both(self) -> Option<(A, B)> {
        match self {
            Both(a, b) => Some((a, b)),
            _ => None,
        }
    }

    /// Converts from `&EitherOrBoth<A, B>` to `EitherOrBoth<&A, &B>`.
    pub fn as_ref(&self) -> EitherOrBoth<&A, &B> {
        match *self {
            Left(ref left) => Left(left),
            Right(ref right) => Right(right),
            Both(ref left, ref right) => Both(left, right),
        }
    }

    /// Converts from `&mut EitherOrBoth<A, B>` to `EitherOrBoth<&mut A, &mut B>`.
    pub fn as_mut(&mut self) -> EitherOrBoth<&mut A, &mut B> {
        match *self {
            Left(ref mut left) => Left(left),
            Right(ref mut right) => Right(right),
            Both(ref mut left, ref mut right) => Both(left, right),
        }
    }

    /// Convert `EitherOrBoth<A, B>` to `EitherOrBoth<B, A>`.
    pub fn flip(self) -> EitherOrBoth<B, A> {
        match self {
            Left(a) => Right(a),
            Right(b) => Left(b),
            Both(a, b) => Both(b, a),
        }
    }

    /// Apply the function `f` on the value `a` in `Left(a)` or `Both(a, b)` variants. If it is
    /// present rewrapping the result in `self`'s original variant.
    pub fn map_left<F, M>(self, f: F) -> EitherOrBoth<M, B>
    where
        F: FnOnce(A) -> M,
    {
        match self {
            Both(a, b) => Both(f(a), b),
            Left(a) => Left(f(a)),
            Right(b) => Right(b),
        }
    }

    /// Apply the function `f` on the value `b` in `Right(b)` or `Both(a, b)` variants.
    /// If it is present rewrapping the result in `self`'s original variant.
    pub fn map_right<F, M>(self, f: F) -> EitherOrBoth<A, M>
    where
        F: FnOnce(B) -> M,
    {
        match self {
            Left(a) => Left(a),
            Right(b) => Right(f(b)),
            Both(a, b) => Both(a, f(b)),
        }
    }

    /// Apply the functions `f` and `g` on the value `a` and `b` respectively;
    /// found in `Left(a)`, `Right(b)`, or `Both(a, b)` variants.
    /// The Result is rewrapped `self`'s original variant.
    pub fn map_any<F, L, G, R>(self, f: F, g: G) -> EitherOrBoth<L, R>
    where
        F: FnOnce(A) -> L,
        G: FnOnce(B) -> R,
    {
        match self {
            Left(a) => Left(f(a)),
            Right(b) => Right(g(b)),
            Both(a, b) => Both(f(a), g(b)),
        }
    }

    /// Apply the function `f` on the value `a` in `Left(a)` or `Both(a, _)` variants if it is
    /// present.
    pub fn left_and_then<F, L>(self, f: F) -> EitherOrBoth<L, B>
    where
        F: FnOnce(A) -> EitherOrBoth<L, B>,
    {
        match self {
            Left(a) | Both(a, _) => f(a),
            Right(b) => Right(b),
        }
    }

    /// Apply the function `f` on the value `b`
    /// in `Right(b)` or `Both(_, b)` variants if it is present.
    pub fn right_and_then<F, R>(self, f: F) -> EitherOrBoth<A, R>
    where
        F: FnOnce(B) -> EitherOrBoth<A, R>,
    {
        match self {
            Left(a) => Left(a),
            Right(b) | Both(_, b) => f(b),
        }
    }

    /// Returns a tuple consisting of the `l` and `r` in `Both(l, r)`, if present.
    /// Otherwise, returns the wrapped value for the present element, and the [`default`](Default::default)
    /// for the other.
    pub fn or_default(self) -> (A, B)
    where
        A: Default,
        B: Default,
    {
        match self {
            EitherOrBoth::Left(l) => (l, B::default()),
            EitherOrBoth::Right(r) => (A::default(), r),
            EitherOrBoth::Both(l, r) => (l, r),
        }
    }
}

impl<T> EitherOrBoth<T, T> {
    /// Return either value of left, right, or the product of `f` applied where `Both` are present.
    pub fn reduce<F>(self, f: F) -> T
    where
        F: FnOnce(T, T) -> T,
    {
        match self {
            Left(a) => a,
            Right(b) => b,
            Both(a, b) => f(a, b),
        }
    }
}

impl<A, B> Into<Option<Either<A, B>>> for EitherOrBoth<A, B> {
    fn into(self) -> Option<Either<A, B>> {
        match self {
            EitherOrBoth::Left(l) => Some(Either::Left(l)),
            EitherOrBoth::Right(r) => Some(Either::Right(r)),
            _ => None,
        }
    }
}
