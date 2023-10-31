use super::impls::{foreach_map_create, foreach_set_create};
use crate::{
    duplicate_key_impls::{
        DuplicateInsertsFirstWinsMap, DuplicateInsertsLastWinsSet, PreventDuplicateInsertsMap,
        PreventDuplicateInsertsSet,
    },
    prelude::*,
    MapFirstKeyWins, MapPreventDuplicates, SetLastValueWins, SetPreventDuplicates,
};
#[cfg(feature = "indexmap_1")]
use indexmap_1::{IndexMap, IndexSet};

struct SetPreventDuplicatesVisitor<SET, T, TAs>(PhantomData<(SET, T, TAs)>);

impl<'de, SET, T, TAs> Visitor<'de> for SetPreventDuplicatesVisitor<SET, T, TAs>
where
    SET: PreventDuplicateInsertsSet<T>,
    TAs: DeserializeAs<'de, T>,
{
    type Value = SET;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    #[inline]
    fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Self::Value::new(access.size_hint());

        while let Some(value) = access.next_element::<DeserializeAsWrap<T, TAs>>()? {
            if !values.insert(value.into_inner()) {
                return Err(DeError::custom("invalid entry: found duplicate value"));
            };
        }

        Ok(values)
    }
}

struct SetLastValueWinsVisitor<SET, T, TAs>(PhantomData<(SET, T, TAs)>);

impl<'de, SET, T, TAs> Visitor<'de> for SetLastValueWinsVisitor<SET, T, TAs>
where
    SET: DuplicateInsertsLastWinsSet<T>,
    TAs: DeserializeAs<'de, T>,
{
    type Value = SET;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    #[inline]
    fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Self::Value::new(access.size_hint());

        while let Some(value) = access.next_element::<DeserializeAsWrap<T, TAs>>()? {
            values.replace(value.into_inner());
        }

        Ok(values)
    }
}

#[cfg(feature = "alloc")]
macro_rules! set_impl {
    (
        $ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)* )* >,
        $with_capacity:expr,
        $append:ident
    ) => {
        impl<'de, T, TAs $(, $typaram)*> DeserializeAs<'de, $ty<T $(, $typaram)*>> for SetPreventDuplicates<TAs>
        where
            TAs: DeserializeAs<'de, T>,
            $(T: $tbound1 $(+ $tbound2)*,)*
            $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize_as<D>(deserializer: D) -> Result<$ty<T $(, $typaram)*>, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_seq(SetPreventDuplicatesVisitor::<$ty<T $(, $typaram)*>, T, TAs>(
                    PhantomData,
                ))
            }
        }

        impl<'de, T, TAs $(, $typaram)*> DeserializeAs<'de, $ty<T $(, $typaram)*>> for SetLastValueWins<TAs>
        where
            TAs: DeserializeAs<'de, T>,
            $(T: $tbound1 $(+ $tbound2)*,)*
            $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize_as<D>(deserializer: D) -> Result<$ty<T $(, $typaram)*>, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer
                    .deserialize_seq(SetLastValueWinsVisitor::<$ty<T $(, $typaram)*>, T, TAs>(PhantomData))
            }
        }
    }
}
foreach_set_create!(set_impl);

struct MapPreventDuplicatesVisitor<MAP, K, KAs, V, VAs>(PhantomData<(MAP, K, KAs, V, VAs)>);

impl<'de, MAP, K, KAs, V, VAs> Visitor<'de> for MapPreventDuplicatesVisitor<MAP, K, KAs, V, VAs>
where
    MAP: PreventDuplicateInsertsMap<K, V>,
    KAs: DeserializeAs<'de, K>,
    VAs: DeserializeAs<'de, V>,
{
    type Value = MAP;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a map")
    }

    #[inline]
    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Self::Value::new(access.size_hint());

        while let Some((key, value)) =
            access.next_entry::<DeserializeAsWrap<K, KAs>, DeserializeAsWrap<V, VAs>>()?
        {
            if !values.insert(key.into_inner(), value.into_inner()) {
                return Err(DeError::custom("invalid entry: found duplicate key"));
            };
        }

        Ok(values)
    }
}

struct MapFirstKeyWinsVisitor<MAP, K, KAs, V, VAs>(PhantomData<(MAP, K, KAs, V, VAs)>);

impl<'de, MAP, K, KAs, V, VAs> Visitor<'de> for MapFirstKeyWinsVisitor<MAP, K, KAs, V, VAs>
where
    MAP: DuplicateInsertsFirstWinsMap<K, V>,
    KAs: DeserializeAs<'de, K>,
    VAs: DeserializeAs<'de, V>,
{
    type Value = MAP;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a map")
    }

    #[inline]
    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Self::Value::new(access.size_hint());

        while let Some((key, value)) =
            access.next_entry::<DeserializeAsWrap<K, KAs>, DeserializeAsWrap<V, VAs>>()?
        {
            values.insert(key.into_inner(), value.into_inner());
        }

        Ok(values)
    }
}

#[cfg(feature = "alloc")]
macro_rules! map_impl {
    (
        $ty:ident < K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)* >,
        $with_capacity:expr
    ) => {
        impl<'de, K, V, KAs, VAs $(, $typaram)*> DeserializeAs<'de, $ty<K, V $(, $typaram)*>>
            for MapPreventDuplicates<KAs, VAs>
        where
            KAs: DeserializeAs<'de, K>,
            VAs: DeserializeAs<'de, V>,
            $(K: $kbound1 $(+ $kbound2)*,)*
            $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize_as<D>(deserializer: D) -> Result<$ty<K, V $(, $typaram)*>, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_map(MapPreventDuplicatesVisitor::<
                    $ty<K, V $(, $typaram)*>,
                    K,
                    KAs,
                    V,
                    VAs,
                >(PhantomData))
            }
        }

        impl<'de, K, V, KAs, VAs $(, $typaram)*> DeserializeAs<'de, $ty<K, V $(, $typaram)*>>
            for MapFirstKeyWins<KAs, VAs>
        where
            KAs: DeserializeAs<'de, K>,
            VAs: DeserializeAs<'de, V>,
            $(K: $kbound1 $(+ $kbound2)*,)*
            $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize_as<D>(deserializer: D) -> Result<$ty<K, V $(, $typaram)*>, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_map(MapFirstKeyWinsVisitor::<$ty<K, V $(, $typaram)*>, K, KAs, V, VAs>(
                    PhantomData,
                ))
            }
        }
    };
}
foreach_map_create!(map_impl);
