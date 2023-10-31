use super::impls::{foreach_map, foreach_set};
use crate::{
    prelude::*, MapFirstKeyWins, MapPreventDuplicates, SetLastValueWins, SetPreventDuplicates,
};
#[cfg(feature = "indexmap_1")]
use indexmap_1::{IndexMap, IndexSet};

macro_rules! set_duplicate_handling {
    ($tyorig:ident < T $(, $typaram:ident : $bound:ident)* >) => {
        impl<T, TAs $(, $typaram)*> SerializeAs<$tyorig<T $(, $typaram)*>> for SetPreventDuplicates<TAs>
        where
            TAs: SerializeAs<T>,
            $($typaram: ?Sized + $bound,)*
        {
            fn serialize_as<S>(value: &$tyorig<T $(, $typaram)*>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                <$tyorig<TAs $(, $typaram)*>>::serialize_as(value, serializer)
            }
        }

        impl<T, TAs $(, $typaram)*> SerializeAs<$tyorig<T $(, $typaram)*>> for SetLastValueWins<TAs>
        where
            TAs: SerializeAs<T>,
            $($typaram: ?Sized + $bound,)*
        {
            fn serialize_as<S>(value: &$tyorig<T $(, $typaram)*>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                <$tyorig<TAs $(, $typaram)*>>::serialize_as(value, serializer)
            }
        }
    }
}
foreach_set!(set_duplicate_handling);

macro_rules! map_duplicate_handling {
    ($tyorig:ident < K, V $(, $typaram:ident : $bound:ident)* >) => {
        impl<K, KAs, V, VAs $(, $typaram)*> SerializeAs<$tyorig<K, V $(, $typaram)*>> for MapPreventDuplicates<KAs, VAs>
        where
            KAs: SerializeAs<K>,
            VAs: SerializeAs<V>,
            $($typaram: ?Sized + $bound,)*
        {
            fn serialize_as<S>(value: &$tyorig<K, V $(, $typaram)*>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                <$tyorig<KAs, VAs $(, $typaram)*>>::serialize_as(value, serializer)
            }
        }

        impl<K, KAs, V, VAs $(, $typaram)*> SerializeAs<$tyorig<K, V $(, $typaram)*>> for MapFirstKeyWins<KAs, VAs>
        where
            KAs: SerializeAs<K>,
            VAs: SerializeAs<V>,
            $($typaram: ?Sized + $bound,)*
        {
            fn serialize_as<S>(value: &$tyorig<K, V $(, $typaram)*>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                <$tyorig<KAs, VAs $(, $typaram)*>>::serialize_as(value, serializer)
            }
        }
    }
}
foreach_map!(map_duplicate_handling);
