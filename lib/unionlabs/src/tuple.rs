pub use macros::AsTuple;

pub trait Tuple {
    type Ref<'a>: Tuple
    where
        Self: 'a;

    const ARITY: usize;
}

pub trait TupleAsRef: Tuple {
    fn as_ref(&self) -> Self::Ref<'_>;
}

pub trait TupleFromRef: Tuple {
    fn from_ref(tuple: Self::Ref<'_>) -> Self;
}

macro_rules! impl_is_tuple {
        ($($T:ident)*) => {
            impl<$($T),*> Tuple for ($($T,)*) {
                type Ref<'a> = ($(&'a $T,)*) where $($T: 'a,)*;

                const ARITY: usize = {
                    $(const $T: usize = 1;)*

                    $($T + )* 0
                };
            }

            impl<$($T),*> TupleAsRef for ($($T,)*) {
                fn as_ref(&self) -> Self::Ref<'_> {
                    #[allow(non_snake_case)]
                    let ($($T,)*) = self;

                    #[allow(clippy::unused_unit)]
                    ($(&$T,)*)
                }
            }

            impl<$($T: Clone),*> TupleFromRef for ($($T,)*) {
                fn from_ref(tuple: Self::Ref<'_>) -> Self {
                    #[allow(non_snake_case)]
                    let ($($T,)*) = tuple;

                    #[allow(clippy::unused_unit)]
                    ($($T.clone(),)*)
                }
            }
        };
    }

impl_is_tuple!();
impl_is_tuple!(A);
impl_is_tuple!(A B);
impl_is_tuple!(A B C);
impl_is_tuple!(A B C D);
impl_is_tuple!(A B C D E);

/// Convert a value between it's tuple form.
///
/// This can be automatically derived on structs:
///
/// ```rust
/// use unionlabs::tuple::AsTuple;
///
/// #[derive(Debug, Clone, PartialEq, AsTuple)]
/// struct Struct {
///     pub a: u64,
///     pub b: String,
/// }
///
/// let s = Struct { a: 1, b: "b".to_owned() };
/// let (a, b): (&u64, &String) = s.as_tuple();
///
/// assert_eq!(a, 1);
/// assert_eq!(b, "b");
///
/// assert_eq!(s, Struct::from_tuple(s.clone().into_tuple()));
/// ```
pub trait AsTuple {
    type Tuple: Tuple + TupleAsRef;

    fn as_tuple(&self) -> <Self::Tuple as Tuple>::Ref<'_>;

    fn into_tuple(self) -> Self::Tuple;

    fn from_tuple(tuple: Self::Tuple) -> Self;
}

impl AsTuple for () {
    type Tuple = ();

    fn as_tuple(&self) -> <Self::Tuple as Tuple>::Ref<'_> {}

    fn into_tuple(self) -> Self::Tuple {}

    fn from_tuple((): Self::Tuple) -> Self {}
}
