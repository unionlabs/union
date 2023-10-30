//! Sources for key-value pairs.

#[cfg(feature = "kv_unstable_sval")]
extern crate sval;
#[cfg(feature = "kv_unstable_sval")]
extern crate sval_ref;

#[cfg(feature = "kv_unstable_serde")]
extern crate serde;

use kv::{Error, Key, ToKey, ToValue, Value};
use std::fmt;

/// A source of key-value pairs.
///
/// The source may be a single pair, a set of pairs, or a filter over a set of pairs.
/// Use the [`Visitor`](trait.Visitor.html) trait to inspect the structured data
/// in a source.
pub trait Source {
    /// Visit key-value pairs.
    ///
    /// A source doesn't have to guarantee any ordering or uniqueness of key-value pairs.
    /// If the given visitor returns an error then the source may early-return with it,
    /// even if there are more key-value pairs.
    ///
    /// # Implementation notes
    ///
    /// A source should yield the same key-value pairs to a subsequent visitor unless
    /// that visitor itself fails.
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>;

    /// Get the value for a given key.
    ///
    /// If the key appears multiple times in the source then which key is returned
    /// is implementation specific.
    ///
    /// # Implementation notes
    ///
    /// A source that can provide a more efficient implementation of this method
    /// should override it.
    #[cfg(not(test))]
    fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
        get_default(self, key)
    }

    #[cfg(test)]
    fn get<'v>(&'v self, key: Key) -> Option<Value<'v>>;

    /// Count the number of key-value pairs that can be visited.
    ///
    /// # Implementation notes
    ///
    /// A source that knows the number of key-value pairs upfront may provide a more
    /// efficient implementation.
    ///
    /// A subsequent call to `visit` should yield the same number of key-value pairs
    /// to the visitor, unless that visitor fails part way through.
    #[cfg(not(test))]
    fn count(&self) -> usize {
        count_default(self)
    }

    #[cfg(test)]
    fn count(&self) -> usize;
}

/// The default implemention of `Source::get`
pub(crate) fn get_default<'v>(source: &'v (impl Source + ?Sized), key: Key) -> Option<Value<'v>> {
    struct Get<'k, 'v> {
        key: Key<'k>,
        found: Option<Value<'v>>,
    }

    impl<'k, 'kvs> Visitor<'kvs> for Get<'k, 'kvs> {
        fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
            if self.key == key {
                self.found = Some(value);
            }

            Ok(())
        }
    }

    let mut get = Get { key, found: None };

    let _ = source.visit(&mut get);
    get.found
}

/// The default implementation of `Source::count`.
pub(crate) fn count_default(source: impl Source) -> usize {
    struct Count(usize);

    impl<'kvs> Visitor<'kvs> for Count {
        fn visit_pair(&mut self, _: Key<'kvs>, _: Value<'kvs>) -> Result<(), Error> {
            self.0 += 1;

            Ok(())
        }
    }

    let mut count = Count(0);
    let _ = source.visit(&mut count);
    count.0
}

impl<'a, T> Source for &'a T
where
    T: Source + ?Sized,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        Source::visit(&**self, visitor)
    }

    fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
        Source::get(&**self, key)
    }

    fn count(&self) -> usize {
        Source::count(&**self)
    }
}

impl<K, V> Source for (K, V)
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        visitor.visit_pair(self.0.to_key(), self.1.to_value())
    }

    fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
        if self.0.to_key() == key {
            Some(self.1.to_value())
        } else {
            None
        }
    }

    fn count(&self) -> usize {
        1
    }
}

impl<S> Source for [S]
where
    S: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        for source in self {
            source.visit(visitor)?;
        }

        Ok(())
    }

    fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
        for source in self {
            if let Some(found) = source.get(key.clone()) {
                return Some(found);
            }
        }

        None
    }

    fn count(&self) -> usize {
        self.len()
    }
}

impl<S> Source for Option<S>
where
    S: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        if let Some(ref source) = *self {
            source.visit(visitor)?;
        }

        Ok(())
    }

    fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
        self.as_ref().and_then(|s| s.get(key))
    }

    fn count(&self) -> usize {
        self.as_ref().map(Source::count).unwrap_or(0)
    }
}

/// A visitor for the key-value pairs in a [`Source`](trait.Source.html).
pub trait Visitor<'kvs> {
    /// Visit a key-value pair.
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error>;
}

impl<'a, 'kvs, T> Visitor<'kvs> for &'a mut T
where
    T: Visitor<'kvs> + ?Sized,
{
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        (**self).visit_pair(key, value)
    }
}

impl<'a, 'b: 'a, 'kvs> Visitor<'kvs> for fmt::DebugMap<'a, 'b> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        self.entry(&key, &value);
        Ok(())
    }
}

impl<'a, 'b: 'a, 'kvs> Visitor<'kvs> for fmt::DebugList<'a, 'b> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        self.entry(&(key, value));
        Ok(())
    }
}

impl<'a, 'b: 'a, 'kvs> Visitor<'kvs> for fmt::DebugSet<'a, 'b> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        self.entry(&(key, value));
        Ok(())
    }
}

impl<'a, 'b: 'a, 'kvs> Visitor<'kvs> for fmt::DebugTuple<'a, 'b> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        self.field(&key);
        self.field(&value);
        Ok(())
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;
    use std::borrow::Borrow;
    use std::collections::{BTreeMap, HashMap};
    use std::hash::{BuildHasher, Hash};

    impl<S> Source for Box<S>
    where
        S: Source + ?Sized,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
            Source::visit(&**self, visitor)
        }

        fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
            Source::get(&**self, key)
        }

        fn count(&self) -> usize {
            Source::count(&**self)
        }
    }

    impl<S> Source for Vec<S>
    where
        S: Source,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
            Source::visit(&**self, visitor)
        }

        fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
            Source::get(&**self, key)
        }

        fn count(&self) -> usize {
            Source::count(&**self)
        }
    }

    impl<'kvs, V> Visitor<'kvs> for Box<V>
    where
        V: Visitor<'kvs> + ?Sized,
    {
        fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
            (**self).visit_pair(key, value)
        }
    }

    impl<K, V, S> Source for HashMap<K, V, S>
    where
        K: ToKey + Borrow<str> + Eq + Hash,
        V: ToValue,
        S: BuildHasher,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
            for (key, value) in self {
                visitor.visit_pair(key.to_key(), value.to_value())?;
            }
            Ok(())
        }

        fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
            HashMap::get(self, key.as_str()).map(|v| v.to_value())
        }

        fn count(&self) -> usize {
            self.len()
        }
    }

    impl<K, V> Source for BTreeMap<K, V>
    where
        K: ToKey + Borrow<str> + Ord,
        V: ToValue,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
            for (key, value) in self {
                visitor.visit_pair(key.to_key(), value.to_value())?;
            }
            Ok(())
        }

        fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
            BTreeMap::get(self, key.as_str()).map(|v| v.to_value())
        }

        fn count(&self) -> usize {
            self.len()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use kv::value::tests::Token;
        use std::collections::{BTreeMap, HashMap};

        #[test]
        fn count() {
            assert_eq!(1, Source::count(&Box::new(("a", 1))));
            assert_eq!(2, Source::count(&vec![("a", 1), ("b", 2)]));
        }

        #[test]
        fn get() {
            let source = vec![("a", 1), ("b", 2), ("a", 1)];
            assert_eq!(
                Token::I64(1),
                Source::get(&source, Key::from_str("a")).unwrap().to_token()
            );

            let source = Box::new(Option::None::<(&str, i32)>);
            assert!(Source::get(&source, Key::from_str("a")).is_none());
        }

        #[test]
        fn hash_map() {
            let mut map = HashMap::new();
            map.insert("a", 1);
            map.insert("b", 2);

            assert_eq!(2, Source::count(&map));
            assert_eq!(
                Token::I64(1),
                Source::get(&map, Key::from_str("a")).unwrap().to_token()
            );
        }

        #[test]
        fn btree_map() {
            let mut map = BTreeMap::new();
            map.insert("a", 1);
            map.insert("b", 2);

            assert_eq!(2, Source::count(&map));
            assert_eq!(
                Token::I64(1),
                Source::get(&map, Key::from_str("a")).unwrap().to_token()
            );
        }
    }
}

/// The result of calling `Source::as_map`.
pub struct AsMap<S>(S);

/// Visit this source as a map.
pub fn as_map<S>(source: S) -> AsMap<S>
where
    S: Source,
{
    AsMap(source)
}

impl<S> Source for AsMap<S>
where
    S: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        self.0.visit(visitor)
    }

    fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
        self.0.get(key)
    }

    fn count(&self) -> usize {
        self.0.count()
    }
}

impl<S> fmt::Debug for AsMap<S>
where
    S: Source,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_map();
        self.0.visit(&mut f).map_err(|_| fmt::Error)?;
        f.finish()
    }
}

/// The result of calling `Source::as_list`
pub struct AsList<S>(S);

/// Visit this source as a list.
pub fn as_list<S>(source: S) -> AsList<S>
where
    S: Source,
{
    AsList(source)
}

impl<S> Source for AsList<S>
where
    S: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        self.0.visit(visitor)
    }

    fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
        self.0.get(key)
    }

    fn count(&self) -> usize {
        self.0.count()
    }
}

impl<S> fmt::Debug for AsList<S>
where
    S: Source,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_list();
        self.0.visit(&mut f).map_err(|_| fmt::Error)?;
        f.finish()
    }
}

#[cfg(feature = "kv_unstable_sval")]
mod sval_support {
    use super::*;

    impl<S> self::sval::Value for AsMap<S>
    where
        S: Source,
    {
        fn stream<'sval, SV: self::sval::Stream<'sval> + ?Sized>(
            &'sval self,
            stream: &mut SV,
        ) -> self::sval::Result {
            struct StreamVisitor<'a, V: ?Sized>(&'a mut V);

            impl<'a, 'kvs, V: self::sval::Stream<'kvs> + ?Sized> Visitor<'kvs> for StreamVisitor<'a, V> {
                fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
                    self.0
                        .map_key_begin()
                        .map_err(|_| Error::msg("failed to stream map key"))?;
                    sval_ref::stream_ref(self.0, key)
                        .map_err(|_| Error::msg("failed to stream map key"))?;
                    self.0
                        .map_key_end()
                        .map_err(|_| Error::msg("failed to stream map key"))?;

                    self.0
                        .map_value_begin()
                        .map_err(|_| Error::msg("failed to stream map value"))?;
                    sval_ref::stream_ref(self.0, value)
                        .map_err(|_| Error::msg("failed to stream map value"))?;
                    self.0
                        .map_value_end()
                        .map_err(|_| Error::msg("failed to stream map value"))?;

                    Ok(())
                }
            }

            stream
                .map_begin(Some(self.count()))
                .map_err(|_| self::sval::Error::new())?;

            self.visit(&mut StreamVisitor(stream))
                .map_err(|_| self::sval::Error::new())?;

            stream.map_end().map_err(|_| self::sval::Error::new())
        }
    }

    impl<S> self::sval::Value for AsList<S>
    where
        S: Source,
    {
        fn stream<'sval, SV: self::sval::Stream<'sval> + ?Sized>(
            &'sval self,
            stream: &mut SV,
        ) -> self::sval::Result {
            struct StreamVisitor<'a, V: ?Sized>(&'a mut V);

            impl<'a, 'kvs, V: self::sval::Stream<'kvs> + ?Sized> Visitor<'kvs> for StreamVisitor<'a, V> {
                fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
                    self.0
                        .seq_value_begin()
                        .map_err(|_| Error::msg("failed to stream seq value"))?;
                    self::sval_ref::stream_ref(self.0, (key, value))
                        .map_err(|_| Error::msg("failed to stream seq value"))?;
                    self.0
                        .seq_value_end()
                        .map_err(|_| Error::msg("failed to stream seq value"))?;

                    Ok(())
                }
            }

            stream
                .seq_begin(Some(self.count()))
                .map_err(|_| self::sval::Error::new())?;

            self.visit(&mut StreamVisitor(stream))
                .map_err(|_| self::sval::Error::new())?;

            stream.seq_end().map_err(|_| self::sval::Error::new())
        }
    }

    #[cfg(test)]
    mod tests {
        extern crate sval_derive;

        use super::*;

        use self::sval_derive::Value;

        use crate::kv::source;

        #[test]
        fn derive_stream() {
            #[derive(Value)]
            pub struct MyRecordAsMap<'a> {
                msg: &'a str,
                kvs: source::AsMap<&'a dyn Source>,
            }

            #[derive(Value)]
            pub struct MyRecordAsList<'a> {
                msg: &'a str,
                kvs: source::AsList<&'a dyn Source>,
            }
        }
    }
}

#[cfg(feature = "kv_unstable_serde")]
pub mod as_map {
    //! `serde` adapters for serializing a `Source` as a map.

    use super::*;

    use self::serde::{Serialize, Serializer};

    /// Serialize a `Source` as a map.
    pub fn serialize<T, S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Source,
        S: Serializer,
    {
        as_map(source).serialize(serializer)
    }
}

#[cfg(feature = "kv_unstable_serde")]
pub mod as_list {
    //! `serde` adapters for serializing a `Source` as a list.

    use super::*;

    use self::serde::{Serialize, Serializer};

    /// Serialize a `Source` as a list.
    pub fn serialize<T, S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Source,
        S: Serializer,
    {
        as_list(source).serialize(serializer)
    }
}

#[cfg(feature = "kv_unstable_serde")]
mod serde_support {
    use super::*;

    use self::serde::ser::{Error as SerError, Serialize, SerializeMap, SerializeSeq, Serializer};

    impl<T> Serialize for AsMap<T>
    where
        T: Source,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            struct SerializerVisitor<'a, S>(&'a mut S);

            impl<'a, 'kvs, S> Visitor<'kvs> for SerializerVisitor<'a, S>
            where
                S: SerializeMap,
            {
                fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
                    self.0
                        .serialize_entry(&key, &value)
                        .map_err(|_| Error::msg("failed to serialize map entry"))?;
                    Ok(())
                }
            }

            let mut map = serializer.serialize_map(Some(self.count()))?;

            self.visit(&mut SerializerVisitor(&mut map))
                .map_err(|_| S::Error::custom("failed to visit key-values"))?;

            map.end()
        }
    }

    impl<T> Serialize for AsList<T>
    where
        T: Source,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            struct SerializerVisitor<'a, S>(&'a mut S);

            impl<'a, 'kvs, S> Visitor<'kvs> for SerializerVisitor<'a, S>
            where
                S: SerializeSeq,
            {
                fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
                    self.0
                        .serialize_element(&(key, value))
                        .map_err(|_| Error::msg("failed to serialize seq entry"))?;
                    Ok(())
                }
            }

            let mut seq = serializer.serialize_seq(Some(self.count()))?;

            self.visit(&mut SerializerVisitor(&mut seq))
                .map_err(|_| S::Error::custom("failed to visit seq"))?;

            seq.end()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use self::serde::Serialize;

        use crate::kv::source;

        #[test]
        fn derive_serialize() {
            #[derive(Serialize)]
            pub struct MyRecordAsMap<'a> {
                msg: &'a str,
                #[serde(flatten)]
                #[serde(with = "source::as_map")]
                kvs: &'a dyn Source,
            }

            #[derive(Serialize)]
            pub struct MyRecordAsList<'a> {
                msg: &'a str,
                #[serde(flatten)]
                #[serde(with = "source::as_list")]
                kvs: &'a dyn Source,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kv::value::tests::Token;

    #[test]
    fn source_is_object_safe() {
        fn _check(_: &dyn Source) {}
    }

    #[test]
    fn visitor_is_object_safe() {
        fn _check(_: &dyn Visitor) {}
    }

    #[test]
    fn count() {
        struct OnePair {
            key: &'static str,
            value: i32,
        }

        impl Source for OnePair {
            fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
                visitor.visit_pair(self.key.to_key(), self.value.to_value())
            }

            fn get<'v>(&'v self, key: Key) -> Option<Value<'v>> {
                get_default(self, key)
            }

            fn count(&self) -> usize {
                count_default(self)
            }
        }

        assert_eq!(1, Source::count(&("a", 1)));
        assert_eq!(2, Source::count(&[("a", 1), ("b", 2)] as &[_]));
        assert_eq!(0, Source::count(&Option::None::<(&str, i32)>));
        assert_eq!(1, Source::count(&OnePair { key: "a", value: 1 }));
    }

    #[test]
    fn get() {
        let source = &[("a", 1), ("b", 2), ("a", 1)] as &[_];
        assert_eq!(
            Token::I64(1),
            Source::get(source, Key::from_str("a")).unwrap().to_token()
        );
        assert_eq!(
            Token::I64(2),
            Source::get(source, Key::from_str("b")).unwrap().to_token()
        );
        assert!(Source::get(&source, Key::from_str("c")).is_none());

        let source = Option::None::<(&str, i32)>;
        assert!(Source::get(&source, Key::from_str("a")).is_none());
    }

    #[test]
    fn as_map() {
        let _ = crate::kv::source::as_map(("a", 1));
        let _ = crate::kv::source::as_map(&("a", 1) as &dyn Source);
    }

    #[test]
    fn as_list() {
        let _ = crate::kv::source::as_list(("a", 1));
        let _ = crate::kv::source::as_list(&("a", 1) as &dyn Source);
    }
}
