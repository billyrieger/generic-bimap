use std::borrow::Borrow;
use std::collections::hash_map;
use std::hash::{BuildHasher, Hash};
use std::iter::FusedIterator;
use std::marker::PhantomData;

use crate::{MapBase, MapExt, MapKind, Ref, Wrapper};

pub struct HashMapKind<S = std::collections::hash_map::RandomState> {
    marker: PhantomData<S>,
}

impl<K, V, S> MapKind<K, V> for HashMapKind<S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    type Map = HashMap<K, V, S>;
}

pub struct HashMap<K, V, S = hash_map::RandomState> {
    map: hash_map::HashMap<Ref<K>, Ref<V>, S>,
}

impl<K, V, S> MapBase for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    type Key = K;
    type Val = V;
    type Iter<'a, K_: 'a, V_: 'a> = Iter<'a, K_, V_> where Self: 'a;

    fn new() -> Self {
        Self {
            map: hash_map::HashMap::default(),
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn iter(&self) -> Self::Iter<'_, Self::Key, Self::Val> {
        Iter {
            iter: self.map.iter(),
        }
    }

    fn insert(&mut self, key: Ref<Self::Key>, val: Ref<Self::Val>) {
        self.map.insert(key, val);
    }
}

impl<K, V, S, Q: ?Sized> MapExt<Q> for HashMap<K, V, S>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
    S: BuildHasher + Default,
{
    fn get(&self, key: &Q) -> Option<&Ref<Self::Val>> {
        self.map.get(Wrapper::wrap(key))
    }

    fn contains(&self, key: &Q) -> bool {
        self.map.contains_key(Wrapper::wrap(key))
    }

    fn remove(&mut self, key: &Q) -> Option<(Ref<Self::Key>, Ref<Self::Val>)> {
        self.map.remove_entry(Wrapper::wrap(key))
    }
}

pub struct Iter<'a, K, V> {
    iter: hash_map::Iter<'a, Ref<K>, Ref<V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a Ref<K>, &'a Ref<V>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}

impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}
