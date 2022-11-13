mod hash_map;

use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

use hash_map::HashMapKind;

/// A reference to a value in a `BiMap`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ref<T> {
    ptr: Rc<T>,
}

impl<T> Ref<T> {
    fn join(x: Ref<T>, y: Ref<T>) -> T {
        // Ensures that x and y are the only two `Rc`s pointing to the
        // allocated value.
        assert!(Rc::ptr_eq(&x.ptr, &y.ptr) && Rc::strong_count(&x.ptr) == 2);
        drop(x);
        Rc::try_unwrap(y.ptr).ok().unwrap()
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.ptr
    }
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
struct Wrapper<T: ?Sized>(T);

impl<T: ?Sized> Wrapper<T> {
    pub fn wrap(value: &T) -> &Self {
        // SAFETY: Wrapper<T> is #[repr(transparent)].
        unsafe { &*(value as *const T as *const Self) }
    }
}

impl<K, Q> Borrow<Wrapper<Q>> for Ref<K>
where
    K: Borrow<Q>,
    Q: ?Sized,
{
    fn borrow(&self) -> &Wrapper<Q> {
        let k: &K = &self.ptr;
        let q: &Q = k.borrow();
        Wrapper::wrap(q)
    }
}

pub trait MapBase {
    type Key;
    type Val;
    type Iter<'a, K: 'a, V: 'a>: Iterator<Item = (&'a Ref<K>, &'a Ref<V>)>
    where
        Self: 'a;

    fn new() -> Self;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn iter(&self) -> Self::Iter<'_, Self::Key, Self::Val>;
    fn insert(&mut self, key: Ref<Self::Key>, val: Ref<Self::Val>);
}

pub trait MapExt<Q: ?Sized = <Self as MapBase>::Key>: MapBase {
    fn get(&self, key: &Q) -> Option<&Ref<Self::Val>>;
    fn contains(&self, key: &Q) -> bool;
    fn remove(&mut self, key: &Q) -> Option<(Ref<Self::Key>, Ref<Self::Val>)>;
}

pub trait Map: MapBase + MapExt {}

impl<T> Map for T where T: MapBase + MapExt {}

pub struct BiMap<LMap, RMap> {
    lmap: LMap,
    rmap: RMap,
}

impl<L, R, LMap, RMap> BiMap<LMap, RMap>
where
    LMap: Map<Key = L, Val = R>,
    RMap: Map<Key = R, Val = L>,
{
    pub fn new() -> Self {
        BiMap {
            lmap: LMap::new(),
            rmap: RMap::new(),
        }
    }

    pub fn get_left<Q: ?Sized>(&self, left: &Q) -> Option<&R>
    where
        LMap: MapExt<Q>,
    {
        self.lmap.get(left).map(|r| &**r)
    }

    pub fn get_right<Q: ?Sized>(&self, right: &Q) -> Option<&L>
    where
        RMap: MapExt<Q>,
    {
        self.rmap.get(right).map(|l| &**l)
    }

    pub fn contains_left<Q: ?Sized>(&self, left: &Q) -> bool
    where
        LMap: MapExt<Q>,
    {
        self.lmap.contains(left)
    }

    pub fn contains_right<Q: ?Sized>(&self, right: &Q) -> bool
    where
        RMap: MapExt<Q>,
    {
        self.rmap.contains(right)
    }

    pub fn remove_left<Q: ?Sized>(&mut self, left: &Q) -> Option<(L, R)>
    where
        LMap: MapExt<Q>,
    {
        let (l0, r0): (Ref<L>, Ref<R>) = self.lmap.remove(left)?;
        let (r1, l1): (Ref<R>, Ref<L>) = self.rmap.remove(&r0).expect("bimap invariant");
        let left = Ref::join(l0, l1);
        let right = Ref::join(r0, r1);
        Some((left, right))
    }

    pub fn remove_right<Q: ?Sized>(&mut self, right: &Q) -> Option<(L, R)>
    where
        RMap: MapExt<Q>,
    {
        let (r0, l0): (Ref<R>, Ref<L>) = self.rmap.remove(right)?;
        let (l1, r1): (Ref<L>, Ref<R>) = self.lmap.remove(&l0).expect("bimap invariant");
        let left = Ref::join(l0, l1);
        let right = Ref::join(r0, r1);
        Some((left, right))
    }
}

pub trait MapKind<K, V> {
    type Map: Map<Key = K, Val = V>;
}

pub type GenericBiMap<L, R, LKind, RKind> =
    BiMap<<LKind as MapKind<L, R>>::Map, <RKind as MapKind<R, L>>::Map>;

pub type BiHashMap<L, R> = GenericBiMap<L, R, HashMapKind, HashMapKind>;
