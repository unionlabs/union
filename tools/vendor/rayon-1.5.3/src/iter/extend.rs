use super::noop::NoopConsumer;
use super::plumbing::{Consumer, Folder, Reducer, UnindexedConsumer};
use super::{IntoParallelIterator, ParallelExtend, ParallelIterator};

use std::borrow::Cow;
use std::collections::LinkedList;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::collections::{BinaryHeap, VecDeque};
use std::hash::{BuildHasher, Hash};

/// Performs a generic `par_extend` by collecting to a `LinkedList<Vec<_>>` in
/// parallel, then extending the collection sequentially.
macro_rules! extend {
    ($self:ident, $par_iter:ident, $extend:ident) => {
        $extend(
            $self,
            $par_iter.into_par_iter().drive_unindexed(ListVecConsumer),
        );
    };
}

/// Computes the total length of a `LinkedList<Vec<_>>`.
fn len<T>(list: &LinkedList<Vec<T>>) -> usize {
    list.iter().map(Vec::len).sum()
}

struct ListVecConsumer;

struct ListVecFolder<T> {
    vec: Vec<T>,
}

impl<T: Send> Consumer<T> for ListVecConsumer {
    type Folder = ListVecFolder<T>;
    type Reducer = ListReducer;
    type Result = LinkedList<Vec<T>>;

    fn split_at(self, _index: usize) -> (Self, Self, Self::Reducer) {
        (Self, Self, ListReducer)
    }

    fn into_folder(self) -> Self::Folder {
        ListVecFolder { vec: Vec::new() }
    }

    fn full(&self) -> bool {
        false
    }
}

impl<T: Send> UnindexedConsumer<T> for ListVecConsumer {
    fn split_off_left(&self) -> Self {
        Self
    }

    fn to_reducer(&self) -> Self::Reducer {
        ListReducer
    }
}

impl<T> Folder<T> for ListVecFolder<T> {
    type Result = LinkedList<Vec<T>>;

    fn consume(mut self, item: T) -> Self {
        self.vec.push(item);
        self
    }

    fn consume_iter<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.vec.extend(iter);
        self
    }

    fn complete(self) -> Self::Result {
        let mut list = LinkedList::new();
        if !self.vec.is_empty() {
            list.push_back(self.vec);
        }
        list
    }

    fn full(&self) -> bool {
        false
    }
}

fn heap_extend<T, Item>(heap: &mut BinaryHeap<T>, list: LinkedList<Vec<Item>>)
where
    BinaryHeap<T>: Extend<Item>,
{
    heap.reserve(len(&list));
    for vec in list {
        heap.extend(vec);
    }
}

/// Extends a binary heap with items from a parallel iterator.
impl<T> ParallelExtend<T> for BinaryHeap<T>
where
    T: Ord + Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        extend!(self, par_iter, heap_extend);
    }
}

/// Extends a binary heap with copied items from a parallel iterator.
impl<'a, T> ParallelExtend<&'a T> for BinaryHeap<T>
where
    T: 'a + Copy + Ord + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = &'a T>,
    {
        extend!(self, par_iter, heap_extend);
    }
}

fn btree_map_extend<K, V, Item>(map: &mut BTreeMap<K, V>, list: LinkedList<Vec<Item>>)
where
    BTreeMap<K, V>: Extend<Item>,
{
    for vec in list {
        map.extend(vec);
    }
}

/// Extends a B-tree map with items from a parallel iterator.
impl<K, V> ParallelExtend<(K, V)> for BTreeMap<K, V>
where
    K: Ord + Send,
    V: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        extend!(self, par_iter, btree_map_extend);
    }
}

/// Extends a B-tree map with copied items from a parallel iterator.
impl<'a, K: 'a, V: 'a> ParallelExtend<(&'a K, &'a V)> for BTreeMap<K, V>
where
    K: Copy + Ord + Send + Sync,
    V: Copy + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = (&'a K, &'a V)>,
    {
        extend!(self, par_iter, btree_map_extend);
    }
}

fn btree_set_extend<T, Item>(set: &mut BTreeSet<T>, list: LinkedList<Vec<Item>>)
where
    BTreeSet<T>: Extend<Item>,
{
    for vec in list {
        set.extend(vec);
    }
}

/// Extends a B-tree set with items from a parallel iterator.
impl<T> ParallelExtend<T> for BTreeSet<T>
where
    T: Ord + Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        extend!(self, par_iter, btree_set_extend);
    }
}

/// Extends a B-tree set with copied items from a parallel iterator.
impl<'a, T> ParallelExtend<&'a T> for BTreeSet<T>
where
    T: 'a + Copy + Ord + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = &'a T>,
    {
        extend!(self, par_iter, btree_set_extend);
    }
}

fn hash_map_extend<K, V, S, Item>(map: &mut HashMap<K, V, S>, list: LinkedList<Vec<Item>>)
where
    HashMap<K, V, S>: Extend<Item>,
    K: Eq + Hash,
    S: BuildHasher,
{
    map.reserve(len(&list));
    for vec in list {
        map.extend(vec);
    }
}

/// Extends a hash map with items from a parallel iterator.
impl<K, V, S> ParallelExtend<(K, V)> for HashMap<K, V, S>
where
    K: Eq + Hash + Send,
    V: Send,
    S: BuildHasher + Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        // See the map_collect benchmarks in rayon-demo for different strategies.
        extend!(self, par_iter, hash_map_extend);
    }
}

/// Extends a hash map with copied items from a parallel iterator.
impl<'a, K: 'a, V: 'a, S> ParallelExtend<(&'a K, &'a V)> for HashMap<K, V, S>
where
    K: Copy + Eq + Hash + Send + Sync,
    V: Copy + Send + Sync,
    S: BuildHasher + Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = (&'a K, &'a V)>,
    {
        extend!(self, par_iter, hash_map_extend);
    }
}

fn hash_set_extend<T, S, Item>(set: &mut HashSet<T, S>, list: LinkedList<Vec<Item>>)
where
    HashSet<T, S>: Extend<Item>,
    T: Eq + Hash,
    S: BuildHasher,
{
    set.reserve(len(&list));
    for vec in list {
        set.extend(vec);
    }
}

/// Extends a hash set with items from a parallel iterator.
impl<T, S> ParallelExtend<T> for HashSet<T, S>
where
    T: Eq + Hash + Send,
    S: BuildHasher + Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        extend!(self, par_iter, hash_set_extend);
    }
}

/// Extends a hash set with copied items from a parallel iterator.
impl<'a, T, S> ParallelExtend<&'a T> for HashSet<T, S>
where
    T: 'a + Copy + Eq + Hash + Send + Sync,
    S: BuildHasher + Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = &'a T>,
    {
        extend!(self, par_iter, hash_set_extend);
    }
}

/// Extends a linked list with items from a parallel iterator.
impl<T> ParallelExtend<T> for LinkedList<T>
where
    T: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        let mut list = par_iter.into_par_iter().drive_unindexed(ListConsumer);
        self.append(&mut list);
    }
}

/// Extends a linked list with copied items from a parallel iterator.
impl<'a, T> ParallelExtend<&'a T> for LinkedList<T>
where
    T: 'a + Copy + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = &'a T>,
    {
        self.par_extend(par_iter.into_par_iter().copied())
    }
}

struct ListConsumer;

struct ListFolder<T> {
    list: LinkedList<T>,
}

struct ListReducer;

impl<T: Send> Consumer<T> for ListConsumer {
    type Folder = ListFolder<T>;
    type Reducer = ListReducer;
    type Result = LinkedList<T>;

    fn split_at(self, _index: usize) -> (Self, Self, Self::Reducer) {
        (Self, Self, ListReducer)
    }

    fn into_folder(self) -> Self::Folder {
        ListFolder {
            list: LinkedList::new(),
        }
    }

    fn full(&self) -> bool {
        false
    }
}

impl<T: Send> UnindexedConsumer<T> for ListConsumer {
    fn split_off_left(&self) -> Self {
        Self
    }

    fn to_reducer(&self) -> Self::Reducer {
        ListReducer
    }
}

impl<T> Folder<T> for ListFolder<T> {
    type Result = LinkedList<T>;

    fn consume(mut self, item: T) -> Self {
        self.list.push_back(item);
        self
    }

    fn consume_iter<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.list.extend(iter);
        self
    }

    fn complete(self) -> Self::Result {
        self.list
    }

    fn full(&self) -> bool {
        false
    }
}

impl<T> Reducer<LinkedList<T>> for ListReducer {
    fn reduce(self, mut left: LinkedList<T>, mut right: LinkedList<T>) -> LinkedList<T> {
        left.append(&mut right);
        left
    }
}

fn flat_string_extend(string: &mut String, list: LinkedList<String>) {
    string.reserve(list.iter().map(String::len).sum());
    string.extend(list);
}

/// Extends a string with characters from a parallel iterator.
impl ParallelExtend<char> for String {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = char>,
    {
        // This is like `extend`, but `Vec<char>` is less efficient to deal
        // with than `String`, so instead collect to `LinkedList<String>`.
        let list = par_iter.into_par_iter().drive_unindexed(ListStringConsumer);
        flat_string_extend(self, list);
    }
}

/// Extends a string with copied characters from a parallel iterator.
impl<'a> ParallelExtend<&'a char> for String {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = &'a char>,
    {
        self.par_extend(par_iter.into_par_iter().copied())
    }
}

struct ListStringConsumer;

struct ListStringFolder {
    string: String,
}

impl Consumer<char> for ListStringConsumer {
    type Folder = ListStringFolder;
    type Reducer = ListReducer;
    type Result = LinkedList<String>;

    fn split_at(self, _index: usize) -> (Self, Self, Self::Reducer) {
        (Self, Self, ListReducer)
    }

    fn into_folder(self) -> Self::Folder {
        ListStringFolder {
            string: String::new(),
        }
    }

    fn full(&self) -> bool {
        false
    }
}

impl UnindexedConsumer<char> for ListStringConsumer {
    fn split_off_left(&self) -> Self {
        Self
    }

    fn to_reducer(&self) -> Self::Reducer {
        ListReducer
    }
}

impl Folder<char> for ListStringFolder {
    type Result = LinkedList<String>;

    fn consume(mut self, item: char) -> Self {
        self.string.push(item);
        self
    }

    fn consume_iter<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        self.string.extend(iter);
        self
    }

    fn complete(self) -> Self::Result {
        let mut list = LinkedList::new();
        if !self.string.is_empty() {
            list.push_back(self.string);
        }
        list
    }

    fn full(&self) -> bool {
        false
    }
}

fn string_extend<Item>(string: &mut String, list: LinkedList<Vec<Item>>)
where
    String: Extend<Item>,
    Item: AsRef<str>,
{
    let len = list.iter().flatten().map(Item::as_ref).map(str::len).sum();
    string.reserve(len);
    for vec in list {
        string.extend(vec);
    }
}

/// Extends a string with string slices from a parallel iterator.
impl<'a> ParallelExtend<&'a str> for String {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = &'a str>,
    {
        extend!(self, par_iter, string_extend);
    }
}

/// Extends a string with strings from a parallel iterator.
impl ParallelExtend<String> for String {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = String>,
    {
        extend!(self, par_iter, string_extend);
    }
}

/// Extends a string with string slices from a parallel iterator.
impl<'a> ParallelExtend<Cow<'a, str>> for String {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = Cow<'a, str>>,
    {
        extend!(self, par_iter, string_extend);
    }
}

fn deque_extend<T, Item>(deque: &mut VecDeque<T>, list: LinkedList<Vec<Item>>)
where
    VecDeque<T>: Extend<Item>,
{
    deque.reserve(len(&list));
    for vec in list {
        deque.extend(vec);
    }
}

/// Extends a deque with items from a parallel iterator.
impl<T> ParallelExtend<T> for VecDeque<T>
where
    T: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        extend!(self, par_iter, deque_extend);
    }
}

/// Extends a deque with copied items from a parallel iterator.
impl<'a, T> ParallelExtend<&'a T> for VecDeque<T>
where
    T: 'a + Copy + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = &'a T>,
    {
        extend!(self, par_iter, deque_extend);
    }
}

fn vec_append<T>(vec: &mut Vec<T>, list: LinkedList<Vec<T>>) {
    vec.reserve(len(&list));
    for mut other in list {
        vec.append(&mut other);
    }
}

/// Extends a vector with items from a parallel iterator.
impl<T> ParallelExtend<T> for Vec<T>
where
    T: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        // See the vec_collect benchmarks in rayon-demo for different strategies.
        let par_iter = par_iter.into_par_iter();
        match par_iter.opt_len() {
            Some(len) => {
                // When Rust gets specialization, we can get here for indexed iterators
                // without relying on `opt_len`.  Until then, `special_extend()` fakes
                // an unindexed mode on the promise that `opt_len()` is accurate.
                super::collect::special_extend(par_iter, len, self);
            }
            None => {
                // This works like `extend`, but `Vec::append` is more efficient.
                let list = par_iter.drive_unindexed(ListVecConsumer);
                vec_append(self, list);
            }
        }
    }
}

/// Extends a vector with copied items from a parallel iterator.
impl<'a, T> ParallelExtend<&'a T> for Vec<T>
where
    T: 'a + Copy + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = &'a T>,
    {
        self.par_extend(par_iter.into_par_iter().copied())
    }
}

/// Collapses all unit items from a parallel iterator into one.
impl ParallelExtend<()> for () {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = ()>,
    {
        par_iter.into_par_iter().drive_unindexed(NoopConsumer)
    }
}
