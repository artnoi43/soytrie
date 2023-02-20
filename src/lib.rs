#![feature(test)]

use std::collections::HashMap;

/// `SearchMode` defines how the trie node treats each match.
pub enum SearchMode {
    /// Entire path has to match, and the final node has to
    /// have some value (i.e. a word).
    Exact,

    /// Entire path has to match, although the final node
    /// maybe non-value (i.e. non-word)
    Prefix,
}

/// `TrieNode<K, V>` represents a node in a trie.
/// If using multiple trie trees, consider using `Trie<K, V>`, which has a root node.
pub struct TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    pub value: Option<V>,

    children: HashMap<K, TrieNode<K, V>>,
}

impl<K, V> TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    /// `new` creates new TrieNode with value set to None
    #[inline]
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            value: None,
        }
    }

    /// `insert_direct_child` returns the mutable reference of the existing child at key `key`,
    /// or if it does not exist, inserts `child` to `self.children` and returning that new child.
    #[inline]
    pub fn insert_direct_child(&mut self, key: K, child: Self) -> &mut Self {
        self.children.entry(key).or_insert(child)
    }

    /// `insert` inserts `value` to child at path `path`. If the child does not exist, a new child
    /// is created and appended to the trie with value `Some(value)`.
    #[inline]
    pub fn insert(&mut self, path: &[K], value: V) {
        let mut curr = self;

        for p in path {
            let next = curr.insert_direct_child(p.clone(), TrieNode::new());
            curr = next;
        }

        curr.value = Some(value);
    }

    /// `get_direct_child` returns a reference to the direct child at key `key`.
    #[inline]
    pub fn get_direct_child(&self, key: K) -> Option<&Self> {
        self.children.get(&key)
    }

    /// `get_direct_child_mut` returns a mutable reference to the direct child at key `key`.
    #[inline]
    pub fn get_direct_child_mut(&mut self, key: K) -> Option<&mut Self> {
        self.children.get_mut(&key)
    }

    /// `search_child` recursively searchs for child at the path, returning reference to it if it exists.
    pub fn search_child(&self, path: &[K]) -> Option<&Self> {
        if path.is_empty() {
            return Some(self);
        }

        self.children
            .get(&path[0])
            .and_then(|child| child.search_child(&path[1..]))
    }

    /// `search_child` recursively searchs for child at the path, returning mutable reference to it if it exists.
    pub fn search_child_mut(&mut self, path: &[K]) -> Option<&mut Self> {
        if path.is_empty() {
            return Some(self);
        }

        self.children
            .get_mut(&path[0])
            .and_then(|child| child.search_child_mut(&path[1..]))
    }

    /// `search` searchs for child at the path, returning boolean value indicating success.
    #[inline]
    pub fn search(&self, mode: SearchMode, path: &[K]) -> bool {
        match self.search_child(path) {
            None => false,
            Some(child) => match mode {
                SearchMode::Prefix => true,
                SearchMode::Exact => child.value.is_some(),
            },
        }
    }

    /// `remove_direct_child` removes and returns the direct owned child at key `key`.
    #[inline]
    pub fn remove_direct_child(&mut self, key: K) -> Option<Self> {
        self.children.remove(&key)
    }

    /// `remove` removes the child at path `path`, returning the owned child.
    #[inline]
    pub fn remove(&mut self, path: &[K]) -> Option<Self> {
        let last_idx = path.len() - 1;

        self.search_child_mut(&path[..last_idx])
            .and_then(|child| child.children.remove(&path[last_idx]))
    }

    /// `collect_children` recursively collects all TrieNode children below `self`
    fn collect_children<'s, 'l>(node: &'l Self, children: &mut Vec<&'s Self>)
    where
        'l: 's,
    {
        children.push(node);
        for child in node.children.values() {
            Self::collect_children(child, children);
        }
    }

    /// `predict` collects all children of the child at path `path`, returning None if the child
    /// does not exist or if the child's number of children is 0.
    #[inline]
    pub fn predict(&self, path: &[K]) -> Option<Vec<&V>> {
        self.search_child(path).and_then(|child| {
            let predicted = child.all_children();
            if predicted.is_empty() {
                return None;
            }

            Some(predicted)
        })
    }

    /// `all_children` returns all children with values as a vector of references to the children.
    #[inline]
    pub fn all_children(&self) -> Vec<&V> {
        let children = &mut Vec::new();
        Self::collect_children(self, children);

        children
            .iter()
            .filter_map(|child| child.value.as_ref())
            .collect()
    }
}

pub struct Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    root: TrieNode<K, V>,
}

/// `Trie<K, V>` wraps an `TrieNode<K, V>` with value `None` as its root node
impl<K, V> Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
        }
    }
}

impl<K, V> From<TrieNode<K, V>> for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn from(node: TrieNode<K, V>) -> Self {
        Self { root: node }
    }
}

impl<K, V> AsRef<TrieNode<K, V>> for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn as_ref(&self) -> &TrieNode<K, V> {
        &self.root
    }
}

impl<K, V> std::ops::Deref for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    type Target = TrieNode<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl<K, V> std::ops::DerefMut for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.root
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;

    use super::*;

    const BENCH_SIZE: usize = 10_000;

    #[test]
    fn test_trie() {
        use super::*;
        let mut trie: Trie<u8, &str> = Trie::new();

        trie.insert(b"a", "a");
        trie.insert(b"ab", "ab");
        trie.insert(b"abc", "abc");
        trie.insert(b"foo", "foo");
        trie.insert(b"foobar", "foobar");
        trie.insert(b"foobar2000", "foobar2000");

        assert_eq!(trie.predict(b"f").expect("bad predict").len(), 3); // foo, foobar, foobar2000
        assert_eq!(trie.predict(b"ab").expect("bad predict").len(), 2); // ab, abc
        assert_eq!(trie.predict(b"fa"), None);

        assert_eq!(trie.search(SearchMode::Prefix, b"a"), true);
        assert_eq!(trie.search(SearchMode::Prefix, b"f"), true);
        assert_eq!(trie.search(SearchMode::Prefix, b"fo"), true);
        assert_eq!(trie.search(SearchMode::Prefix, b"fa"), false);
        assert_eq!(trie.search(SearchMode::Prefix, b"bar"), false);
        assert_eq!(trie.search(SearchMode::Prefix, b"ob"), false);
        assert_eq!(trie.search(SearchMode::Prefix, b"foooba"), false);

        assert_eq!(trie.search(SearchMode::Exact, b"f"), false);
        assert_eq!(trie.search(SearchMode::Exact, b"fo"), false);
        assert_eq!(trie.search(SearchMode::Exact, b"foo"), true);
        assert_eq!(trie.search(SearchMode::Exact, b"foob"), false);
        assert_eq!(trie.search(SearchMode::Exact, b"fooba"), false);
        assert_eq!(trie.search(SearchMode::Exact, b"foobar"), true);

        assert_eq!(trie.all_children().len(), 6);
        assert_eq!(trie.predict(b"a").expect("a node is None").len(), 3);
        assert_eq!(trie.predict(b"f").expect("f node is None").len(), 3);

        let foob_node = trie.root.search_child(b"foob");
        assert_eq!(
            foob_node.expect("foob node is None").all_children().len(),
            2
        );

        let foobar2000_node = trie.search_child(b"foobar2000");
        assert_eq!(
            foobar2000_node
                .expect("foobar2000 node is None")
                .all_children()
                .len(),
            1,
        );

        let foobar2000_node = trie.remove(b"foobar2000").expect("foobar2000 node is None");
        assert_eq!(foobar2000_node.all_children().len(), 1);
        assert_eq!(foobar2000_node.value, Some("foobar2000"));
        assert_eq!(trie.all_children().len(), 5);
        trie.remove(b"abc"); // deletes abc
        assert_eq!(trie.all_children().len(), 4);
        trie.remove(b"ab"); // deletes ab
        assert_eq!(trie.all_children().len(), 3);
        trie.remove(b"ab"); // deletes ab
        assert_eq!(trie.all_children().len(), 3);
        trie.remove(b"f"); // deletes f, fo, foo
        assert_eq!(trie.all_children().len(), 1);
        trie.remove(b"a"); // deletes a
        assert_eq!(trie.all_children().len(), 0);
    }

    fn insert_root(i: usize, b: &mut Bencher, trie: &mut Trie<usize, &str>) {
        b.iter(|| {
            let inputs: Vec<usize> = (i..BENCH_SIZE).into_iter().collect();

            trie.insert(&inputs, "foo");

            assert_eq!(trie.all_children().len(), i + 1);
            assert_eq!(
                trie.search_child(&inputs).and_then(|child| child.value),
                Some("foo")
            );
        })
    }

    #[bench]
    fn insert_bench(b: &mut Bencher) {
        let mut trie = Trie::new();
        (0..3).for_each(|i| insert_root(i, b, &mut trie))
    }
}