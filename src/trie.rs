use std::collections::HashMap;

/// Defines how the trie node treats each match.
#[derive(Clone)]
pub enum SearchMode {
    /// Exact gets valued node
    Exact,
    /// Prefix gets both path and valued nodes
    Prefix,
}

/// A trie node backed by [HashMap](HashMap).
/// In this trie implementation, a node can be either a _valued node_, or a _path node_.
/// A valued node has [`Some(_)`](Some) in the value field, while path node has [`None`](None).
/// Fields `value` and `children` are uncorrelated and can be used arbitarily.
/// If using multiple tries, consider using [`Trie<K, V>`](Trie), which has a path node as root.
pub struct TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    pub value: Option<V>,

    children: HashMap<K, TrieNode<K, V>>,
}

/// # Non-`Clone` implementation usage examples
/// ```
/// # use soytrie::TrieNode;
/// let mut root: TrieNode<u8, &str> = TrieNode::new(); // Creates a root node with value None
/// let node = TrieNode::from("foo"); // Creates a node with value Some("foo")
/// root.insert_child(b"foo", node);
/// root.insert_value(b"foobar", "foobar");
/// root.insert_child(b"baz", "baz".into()); // TrieNode also implements From<T>
///
/// assert!(root.get_direct_child(b'a').is_none());
///
/// {
///     let f_node = root.get_child_mut(b"f").unwrap();
///     f_node.insert_child(b"a", "fa".into());
/// }
///
/// assert_eq!(root.all_children_values().len(), 4); // "baz", "fa", "foo", "foobar"
/// assert_eq!(root.predict(b"f").unwrap().len(), 3); // "fa", "foo", "foobar"
///
/// root.remove(b"fo"); // deletes "foo", "foobar"
/// assert_eq!(root.all_children_values().len(), 2); // "baz", "fa"
///
/// {
///     let f_node = root.get_direct_child_mut(b'f').unwrap();
///     assert_eq!(f_node.all_children_values().len(), 1); // "fa"
///     f_node.insert_value(b"z", "fz");
///     assert_eq!(f_node.all_children_values().len(), 2); // "fa" "fz"
/// }
///
/// assert_eq!(root.all_children_values().len(), 3); // "baz", "fa", "fz"
/// ```
impl<K, V> TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    /// Creates new node with value set to [None](None).
    /// If you want to create a node from a value, use [`from`](From) instead:
    /// ```
    /// # use soytrie::TrieNode;
    /// let mut root = TrieNode::new();
    /// let node = TrieNode::from("foo"); // Creates a node with value Some("foo")
    ///
    /// root.insert_child(b"foo", node);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            value: None,
        }
    }

    /// Returns the mutable reference of the existing child at key `key`.
    /// If it does not exist, inserts `child` to `self.children` and returning that new child.
    #[inline]
    pub fn get_insert_direct_value<T>(&mut self, key: K, child: T) -> &mut Self
    where
        T: Into<Self>,
    {
        self.children.entry(key).or_insert(child.into())
    }

    /// Returns the mutable reference of the existing child at key `key`.
    /// If it does not exist, inserts `child` to `self.children` and returning that new child.
    #[inline]
    pub fn get_insert_direct_child(&mut self, key: K, child: Self) -> &mut Self {
        self.children.entry(key).or_insert(child)
    }

    /// Inserts `child` at path `path`. If the child already exists, it is assigned a completely new value
    /// and 0 children. This means that `insert_child` destroys the existing node, so if you want
    /// to keep the children but change the value when inserting, use
    /// [`insert_value`](Self::insert_value) instead.
    /// Note: The implementation does not use recursion, so deep insertion will not cost long call stacks.
    /// ```
    /// # use soytrie::TrieNode;
    /// let mut node = TrieNode::new();
    /// node.insert_child(b"a", "a".into()); // This adds valued node at "a"
    /// node.insert_child(b"ab", "ab".into()); // This adds valued node at "b"
    /// node.insert_child(b"abcde", "abcde".into()); // This adds valued node at "b"
    ///
    /// assert_eq!(node.all_valued_children().len(), 3); // valued nodes: "a", "b", "e"
    /// assert_eq!(node.all_children().len(), 6); // valued nodes: "a", "b", "e"
    ///
    /// node.insert_child(b"ab", "ab".into()); // This call to insert_child drops the old value at
    /// // "b" and adds a fresh, new valued node at "b". The old "b" node's children was dropped.
    ///
    /// assert_eq!(node.all_valued_children().len(), 2); // valued nodes: "a", "b"
    /// ```
    pub fn insert_child(&mut self, path: &[K], child: Self) {
        // if path.is_empty() {
        //     *self = child;
        //     return;
        // }
        //
        // self.children
        //     .entry(path[0].clone())
        //     .or_insert(Self::new())
        //     .insert_child(&path[1..], child);

        let mut curr = self;
        for p in path {
            let next = curr.get_insert_direct_value(p.clone(), Self::new());
            curr = next;
        }

        *curr = child;
    }

    /// Inserts `value` to child at path `path`. If the child already exists, insert_value changes
    /// only the `value` field of the child. If the child does not exist, a new child
    /// is created and appended to the trie with value `Some(value)`.
    /// ```
    /// # use soytrie::TrieNode;
    /// let mut node = TrieNode::new();
    ///
    /// node.insert_value(b"a", "a"); // This adds valued node at "a"
    /// node.insert_value(b"ab", "ab"); // This adds valued node at "b"
    /// node.insert_value(b"abcde", "abcde"); // This adds path nodes at "c", "d", and valued node at "e"
    ///
    /// assert_eq!(node.all_valued_children().len(), 3); // valued nodes: "a", "b", "e"
    ///
    /// node.insert_child(b"ab", "ab".into()); // This call to insert_child drops the old value at
    /// // "b" and adds a fresh, new valued node at "b". The old "b" node's children was dropped.
    ///
    /// assert_eq!(node.all_valued_children().len(), 2); // valued nodes: "a", "b"
    /// ```
    #[inline]
    pub fn insert_value(&mut self, path: &[K], value: V) {
        let mut curr = self;
        for p in path {
            let next = curr.get_insert_direct_value(p.clone(), Self::new());
            curr = next;
        }

        curr.value = Some(value);
    }

    /// Returns a reference to the direct child at key `key`.
    #[inline(always)]
    pub fn get_direct_child(&self, key: K) -> Option<&Self> {
        self.children.get(&key)
    }

    /// Returns a mutable reference to the direct child at key `key`.
    #[inline(always)]
    pub fn get_direct_child_mut(&mut self, key: K) -> Option<&mut Self> {
        self.children.get_mut(&key)
    }

    /// Recursively searchs for child at the path, returning reference to the child if it exists.
    pub fn get_child(&self, path: &[K]) -> Option<&Self> {
        path.is_empty().then_some(self).or_else(|| {
            self.children
                .get(&path[0])
                .and_then(|child| child.get_child(&path[1..]))
        })
    }

    /// Recursively searchs for child at the path, returning mutable reference to the child if it exists.
    pub fn get_child_mut(&mut self, path: &[K]) -> Option<&mut Self> {
        if path.is_empty() {
            return Some(self);
        }

        self.children
            .get_mut(&path[0])
            .and_then(|child| child.get_child_mut(&path[1..]))
    }

    /// Searchs for child at the path, returning boolean value indicating success.
    #[inline(always)]
    pub fn search(&self, mode: SearchMode, path: &[K]) -> bool {
        match self.get_child(path) {
            None => false,
            Some(child) => match mode {
                SearchMode::Prefix => true,
                SearchMode::Exact => child.value.is_some(),
            },
        }
    }

    /// Removes and returns the direct owned child at key `key`.
    #[inline(always)]
    pub fn remove_direct_child(&mut self, key: K) -> Option<Self> {
        self.children.remove(&key)
    }

    /// Removes the child at path `path`, returning the owned child.
    pub fn remove(&mut self, path: &[K]) -> Option<Self> {
        let last_idx = path.len() - 1;

        self.get_child_mut(&path[..last_idx])
            .and_then(|child| child.children.remove(&path[last_idx]))
    }

    /// Recursively collects all extant children of `node`.
    fn collect_children<'s, 'l>(node: &'l Self, children: &mut Vec<&'s Self>)
    where
        'l: 's,
    {
        children.push(node);

        node.children
            .values()
            .for_each(|child| Self::collect_children(child, children));
    }

    /// Returns all children of the node.
    /// ```
    /// # use soytrie::TrieNode;
    /// let mut node = TrieNode::new();
    ///
    /// node.insert_value(b"a", "a"); // Adds valued node at "a"
    /// node.insert_value(b"ab", "ab"); // Adds valued node at "b"
    /// node.insert_value(b"abcde", "abcde"); // Adds path nodes at "c", "d", and valued node at "e"
    /// node.insert_value(b"xyz", "xyz"); // Adds path nodes at "x", "y", and valued node at "z"
    ///
    /// assert_eq!(node.all_children().len(), 9); // nodes: a, b, c, d, e, x, y, z and root node
    /// ```
    pub fn all_children(&self) -> Vec<&Self> {
        let mut children = Vec::new();
        Self::collect_children(self, &mut children);

        children
    }

    /// Returns all valued child nodes of the node.
    /// ```
    /// # use soytrie::TrieNode;
    /// let mut node = TrieNode::new();
    ///
    /// node.insert_value(b"a", "a"); // This adds valued node at "a"
    /// node.insert_value(b"ab", "ab"); // This adds valued node at "b"
    /// node.insert_value(b"abcde", "abcde"); // This adds path nodes at "c", "d", and valued node at "e"
    /// node.insert_value(b"xyz", "xyz"); // Adds path nodes at "x", "y", and valued node at "z"
    ///
    /// assert_eq!(node.all_valued_children().len(), 4); // valued nodes: "a", "b", "e", and "z"
    /// ```
    pub fn all_valued_children(&self) -> Vec<&Self> {
        self.all_children()
            .into_iter()
            .filter(|child| child.value.is_some())
            .collect()
    }

    /// Returns all values of valued children as a vector of references to the children.
    /// ```
    /// use soytrie::TrieNode;
    /// let mut node = TrieNode::new();
    ///
    /// node.insert_value(b"abc", "abc"); // This adds path nodes at "a" and "b", and valued node at "c"
    /// node.insert_value(b"xyz", "xyz"); // Adds path nodes at "x", "y", and valued node at "z"
    ///
    /// assert_eq!(node.all_valued_children().len(), 2); // valued nodes: "abc", "xyz"
    /// ```
    #[inline]
    pub fn all_children_values(&self) -> Vec<&V> {
        self.all_children()
            .iter()
            .filter_map(|child| child.value.as_ref())
            .collect()
    }

    /// Collects all values of the children of the child at path `path`, returning [`None`](None)
    /// if the child does not exist or if the child's number of children is 0. Otherwise, the
    /// references to values is collected as [`Some(Vec<&V>)`](Some).
    #[inline]
    pub fn predict(&self, path: &[K]) -> Option<Vec<&V>> {
        self.get_child(path).and_then(|child| {
            let predicted = child.all_children_values();

            if predicted.is_empty() {
                return None;
            }

            Some(predicted)
        })
    }
}

/// This impl defines extra methods for [TrieNode<K, V>](TrieNode) where `V: Clone`. It methods in here
/// will receive or return a cloned value that had no reference to the previous parent trie.
/// ```
/// use soytrie::TrieNode;
///
/// let mut node = TrieNode::<u8, u8>::new();
/// node.insert_child(b"1", b'1'.into());
/// let mut cloned = node.get_child_clone(b"1").expect("no such child");
/// cloned.insert_child(b"2", b'2'.into());
///
/// // '2' was not insert into node's trie, only cloned's trie
///
/// assert_eq!(
///     node.all_children_values().len(),
///     1,
/// );
///
/// assert_eq!(
///     cloned.all_children_values().len(),
///     2,
/// );
/// ```
impl<K, V> TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    /// Returns cloned child at key `key`.
    #[inline]
    pub fn get_direct_child_clone(&self, key: K) -> Option<Self> {
        self.children
            .get(&key)
            .and_then(|child| Some(child.clone()))
    }

    /// Returns cloned child at path `path`.
    pub fn get_child_clone(&self, path: &[K]) -> Option<Self> {
        path.is_empty().then_some(self.clone()).or_else(|| {
            self.children
                .get(&path[0])
                .and_then(|child| child.get_child_clone(&path[1..]))
        })
    }
}

/// Creates a valued [node](TrieNode) using [`Some(_)`](Some)
/// without children. Only the [value field](TrieNode::value) is populated.
/// ```
/// # use soytrie::TrieNode;
/// # use std::collections::HashMap;
/// let node: TrieNode<u8, _> = "node".to_string().into();
/// assert_eq!(node.value, Some("node".to_string()));
/// ```
impl<K, V> From<V> for TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn from(value: V) -> Self {
        Self {
            value: Some(value),
            children: HashMap::new(),
        }
    }
}

/// Creates a node from [`Option<V>`](Option) without wrapping it in another [`Some(Some(_))`](Some).
///```
/// # use soytrie::TrieNode;
/// # use std::collections::HashMap;
/// let node: TrieNode<u8, _> = "node".to_string().into();
/// assert!(node == TrieNode::from(Some("node".to_string())));
/// ```
impl<K, V> From<Option<V>> for TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn from(opt: Option<V>) -> Self {
        Self {
            value: opt,
            children: HashMap::new(),
        }
    }
}

/// PartialEq for [TrieNode<K, V>](TrieNode) compares the [value field](TrieNode::value)
/// for equality. As of now, the node's children is not taken into consideration.
/// ```
/// # use soytrie::TrieNode;
/// # use std::collections::HashMap;
/// let mut node1: TrieNode<u8, _> = "node".to_string().into();
/// node1.insert_value(b"e", "e".to_string());
///
/// let mut node2: TrieNode<u8, _> = "node".to_string().into();
/// node2.insert_value(b"f", "f".to_string());
///
/// assert!(node1 == node2) // ok, because we only compare values
/// ```
impl<K, V> PartialEq<Self> for TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.value == rhs.value
    }
}

/// `Debug` for [`TrieNode`](TrieNode) is quite expensive - the node will call
/// [`self.all_children_values`](TrieNode::all_children) to traverse the entire trie and
/// collect all valued children of `self`. It should only be used when debugging.
impl<K, V> std::fmt::Debug for TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash + std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("TrieNode")
            .field("children", &self.all_children_values())
            .field("value", &self.value)
            .finish()
    }
}

/// If `K` and `V` is Clone, then `TrieNode<K, V>` is also `Clone`.
impl<K, V> Clone for TrieNode<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            children: self.children.clone(),
        }
    }
}

pub struct Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    root: TrieNode<K, V>,
}

/// Wraps a [`TrieNode<K, V>`](TrieNode) with value [`None`](None) as its root node.
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

// Clones self's root to new trie root.
impl<K, V> Clone for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
        }
    }
}

// Consumes the node as root for new trie
impl<K, V> From<TrieNode<K, V>> for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn from(node: TrieNode<K, V>) -> Self {
        Self { root: node }
    }
}

// Returns the reference to root node
impl<K, V> AsRef<TrieNode<K, V>> for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn as_ref(&self) -> &TrieNode<K, V> {
        &self.root
    }
}

// Returns the mutable reference to root node
impl<K, V> AsMut<TrieNode<K, V>> for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn as_mut(&mut self) -> &mut TrieNode<K, V> {
        &mut self.root
    }
}

// Derefs to the root node
impl<K, V> std::ops::Deref for Trie<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    type Target = TrieNode<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

// DerefMuts to the root node
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
    #[test]
    fn test_trie() {
        use super::*;

        let mut trie: Trie<u8, &str> = Trie::new();
        trie.insert_value(b"a", "a");
        trie.insert_value(b"ab", "ab");
        trie.insert_value(b"abc", "abc");
        trie.insert_value(b"foo", "foo");
        trie.insert_value(b"foobar", "foobar");
        trie.insert_value(b"foobar2000", "foobar2000");

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

        assert_eq!(trie.all_children_values().len(), 6);
        assert_eq!(trie.predict(b"a").expect("a node is None").len(), 3);
        assert_eq!(trie.predict(b"f").expect("f node is None").len(), 3);

        let foob_node = trie.root.get_child(b"foob");
        assert_eq!(
            foob_node
                .expect("foob node is None")
                .all_children_values()
                .len(),
            2
        );

        let foobar2000_node = trie.get_child(b"foobar2000");
        assert_eq!(
            foobar2000_node
                .expect("foobar2000 node is None")
                .all_children_values()
                .len(),
            1,
        );

        let foobar2000_node = trie.remove(b"foobar2000").expect("foobar2000 node is None");
        assert_eq!(foobar2000_node.all_children_values().len(), 1);
        assert_eq!(foobar2000_node.value, Some("foobar2000"));

        assert_eq!(trie.all_children_values().len(), 5);
        trie.remove(b"abc"); // deletes abc
        assert_eq!(trie.all_children_values().len(), 4);
        trie.remove(b"ab"); // deletes ab
        assert_eq!(trie.all_children_values().len(), 3);
        trie.remove(b"ab"); // deletes ab
        assert_eq!(trie.all_children_values().len(), 3);
        trie.remove(b"f"); // deletes f, fo, foo
        assert_eq!(trie.all_children_values().len(), 1);
        trie.remove(b"a"); // deletes a
        assert_eq!(trie.all_children_values().len(), 0);
    }
}
