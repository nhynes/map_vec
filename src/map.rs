use alloc::{collections::TryReserveError, vec::Vec};
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    iter::FusedIterator,
};

/// `Map` is a data structure with a [`HashMap`]-like API but based on a `Vec`.
///
/// It's primarily useful when you care about constant factors or prefer determinism to speed.
/// Please refer to the docs for [`HashMap`] for details and examples of the Map API.
///
/// ## Example
///
/// ```
/// let mut map = map_vec::Map::new();
/// map.insert("hello".to_string(), "world".to_string());
/// map.entry("hello".to_string()).and_modify(|mut v| v.push_str("!"));
/// assert_eq!(map.get("hello").map(String::as_str), Some("world!"))
/// ```
///
/// [`HashMap`]: std::collections::HashMap
#[derive(Clone, PartialEq, Eq)]
pub struct Map<K, V> {
    backing: Vec<(K, V)>,
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self {
            backing: Vec::default(),
        }
    }
}

impl<K: Eq, V> Map<K, V> {
    pub fn new() -> Self {
        Self {
            backing: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            backing: Vec::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.backing.capacity()
    }

    pub fn clear(&mut self) {
        self.backing.clear()
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.keys().any(|k| key.eq(k.borrow()))
    }

    pub fn drain(&mut self) -> alloc::vec::Drain<(K, V)> {
        self.backing.drain(..)
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        match self.backing.iter_mut().position(|(k, _)| *k == key) {
            Some(pos) => Entry::Occupied(OccupiedEntry {
                entry_pos: pos,
                // entry: unsafe { core::mem::transmute::<&mut (K, V), &'a mut (K, V)>(entry) },
                /* ^ since the only operations on an OccupiedEntry modify `v` in-place, the Vec will
                 * never move in memory (reallocate), so the ref is valid for the duration of the OE. */
                backing: &mut self.backing,
            }),
            None => Entry::Vacant(VacantEntry {
                key,
                backing: &mut self.backing,
            }),
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.backing
            .iter()
            .find(|(k, _)| key.eq(k.borrow()))
            .map(|(_, v)| v)
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.backing
            .iter()
            .find(|(k, _)| key.eq(k.borrow()))
            .map(|(k, v)| (k, v))
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.backing
            .iter_mut()
            .find(|(k, _)| key.eq(k.borrow()))
            .map(|(_, v)| v)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.get_mut(&key) {
            Some(v) => Some(core::mem::replace(v, value)),
            None => {
                self.backing.push((key, value));
                None
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.backing.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.backing.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.backing.iter_mut(),
        }
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { iter: self.iter() }
    }

    pub fn len(&self) -> usize {
        self.backing.len()
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.backing
            .iter()
            .position(|(k, _)| key.eq(k.borrow()))
            .map(|pos| self.backing.swap_remove(pos))
    }

    pub fn reserve(&mut self, additional: usize) {
        self.backing.reserve(additional);
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.backing.retain_mut(|(k, v)| f(k, v));
    }

    pub fn shrink_to_fit(&mut self) {
        self.backing.shrink_to_fit();
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values { iter: self.iter() }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.iter_mut(),
        }
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.backing.shrink_to(min_capacity)
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.backing.try_reserve(additional)
    }
}

impl<K: Debug, V: Debug> fmt::Debug for Map<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.backing.iter().map(|(ref k, ref v)| (k, v)))
            .finish()
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        Iter {
            iter: self.backing.iter(),
        }
    }
}

impl<'a, K, V> IntoIterator for &'a mut Map<K, V> {
    type Item = (&'a mut K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        IterMut {
            iter: self.backing.iter_mut(),
        }
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = alloc::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.backing.into_iter()
    }
}

impl<K: Eq, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let mut this = match iter.size_hint() {
            (min, Some(max)) if min > 0 && min == max => {
                // Exact size is known. Reserve the space.
                Self::with_capacity(min)
            }
            (min, Some(_)) | (min, None) if min > 0 => {
                // The exact size is not known, but there's a minimum size known.
                // We'll reserve what we know.
                Self::with_capacity(min)
            }
            (_, _) => {
                // There isn't even a minimum size known.
                Self::new()
            }
        };

        this.extend(iter);
        this.shrink_to_fit();
        this
    }
}

impl<K: Eq, V> Extend<(K, V)> for Map<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, K: 'a + Copy + Eq, V: 'a + Copy> Extend<(&'a K, &'a V)> for Map<K, V> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(*k, *v);
        }
    }
}

impl<K: Eq, V, T: Into<Vec<(K, V)>>> From<T> for Map<K, V> {
    fn from(values: T) -> Self {
        let values = values.into();
        let mut map = Self::with_capacity(values.len());
        map.extend(values);
        map.shrink_to_fit();
        map
    }
}

impl<Q: Eq + ?Sized, K: Eq + Borrow<Q>, V> core::ops::Index<&Q> for Map<K, V> {
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

#[derive(Debug, Clone)]
pub struct Keys<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<K, V> Keys<'_, K, V> {
    fn map_item<'a>(item: (&'a K, &'a V)) -> &'a K {
        item.0
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::map_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Keys<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::map_item)
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {}
impl<K, V> FusedIterator for Keys<'_, K, V> {}

#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "nightly")))]
#[cfg(feature = "nightly")]
unsafe impl<K, V> core::iter::TrustedLen for Keys<'_, K, V> {}

#[derive(Debug, Clone)]
pub struct Values<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<K, V> Values<'_, K, V> {
    fn map_item<'a>(item: (&'a K, &'a V)) -> &'a V {
        item.1
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::map_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Values<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::map_item)
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {}
impl<K, V> FusedIterator for Values<'_, K, V> {}

#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "nightly")))]
#[cfg(feature = "nightly")]
unsafe impl<K, V> core::iter::TrustedLen for Values<'_, K, V> {}

#[derive(Debug)]
pub struct ValuesMut<'a, K, V> {
    iter: IterMut<'a, K, V>,
}

impl<K, V> ValuesMut<'_, K, V> {
    fn map_item<'a>(item: (&'a mut K, &'a mut V)) -> &'a mut V {
        item.1
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::map_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for ValuesMut<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::map_item)
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {}
impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "nightly")))]
#[cfg(feature = "nightly")]
unsafe impl<K, V> core::iter::TrustedLen for ValuesMut<'_, K, V> {}

#[derive(Debug, Clone)]
pub struct Iter<'a, K, V> {
    iter: core::slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Iter<'a, K, V> {
    fn map_item(item: &'a (K, V)) -> (&'a K, &'a V) {
        (&item.0, &item.1)
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::map_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::map_item)
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}
impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "nightly")))]
#[cfg(feature = "nightly")]
unsafe impl<'a, K, V> core::iter::TrustedLen for Iter<'a, K, V> {}

#[derive(Debug)]
pub struct IterMut<'a, K, V> {
    iter: core::slice::IterMut<'a, (K, V)>,
}

impl<'a, K, V> IterMut<'a, K, V> {
    fn map_item(item: &'a mut (K, V)) -> (&'a mut K, &'a mut V) {
        (&mut item.0, &mut item.1)
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a mut K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::map_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::map_item)
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}
impl<'a, K, V> FusedIterator for IterMut<'a, K, V> {}

#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "nightly")))]
#[cfg(feature = "nightly")]
unsafe impl<'a, K, V> core::iter::TrustedLen for IterMut<'a, K, V> {}

pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V> {
    pub fn and_modify(mut self, f: impl FnOnce(&mut V)) -> Self {
        if let Entry::Occupied(oe) = &mut self {
            f(oe.get_mut())
        }
        self
    }

    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(oe) => oe.key(),
            Entry::Vacant(ve) => ve.key(),
        }
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(oe) => oe.into_mut(),
            Entry::Vacant(ve) => ve.insert(default),
        }
    }

    pub fn or_insert_with(self, f: impl FnOnce() -> V) -> &'a mut V {
        match self {
            Entry::Occupied(oe) => oe.into_mut(),
            Entry::Vacant(ve) => ve.insert(f()),
        }
    }
}

impl<'a, K: 'a, V: Default> Entry<'a, K, V> {
    pub fn or_default(self) -> &'a mut V {
        #[allow(
            clippy::unwrap_or_default,
            // reason = "We can't call the suggested `.or_default()` here \
            //     because we're implementing it."
        )]
        self.or_insert(V::default())
    }
}

pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    entry_pos: usize,
    backing: &'a mut Vec<(K, V)>,
}

impl<'a, K: 'a, V: 'a> OccupiedEntry<'a, K, V> {
    pub fn get(&self) -> &V {
        &self.backing[self.entry_pos].1
    }

    pub fn get_mut(&mut self) -> &mut V {
        &mut self.backing[self.entry_pos].1
    }

    pub fn insert(&mut self, value: V) -> V {
        core::mem::replace(self.get_mut(), value)
    }

    pub fn into_mut(self) -> &'a mut V {
        &mut self.backing[self.entry_pos].1
    }

    pub fn key(&self) -> &K {
        &self.backing[self.entry_pos].0
    }

    pub fn remove(self) -> V {
        self.backing.remove(self.entry_pos).1
    }
}

pub struct VacantEntry<'a, K: 'a, V: 'a> {
    key: K,
    backing: &'a mut Vec<(K, V)>,
}

impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
    pub fn insert(self, value: V) -> &'a mut V {
        self.backing.push((self.key, value));
        &mut self.backing.last_mut().unwrap().1
    }

    pub fn into_key(self) -> K {
        self.key
    }

    pub fn key(&self) -> &K {
        &self.key
    }
}

#[cfg(feature = "serde")]
mod map_serde {
    use core::{fmt, marker::PhantomData};

    use serde::{
        de::{Deserialize, Deserializer, MapAccess, Visitor},
        ser::{Serialize, SerializeMap, Serializer},
    };

    use super::Map;

    #[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "serde")))]
    impl<K, V> Serialize for Map<K, V>
    where
        K: Serialize + Eq,
        V: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(self.len()))?;
            for (k, v) in self {
                map.serialize_entry(k, v)?;
            }
            map.end()
        }
    }

    #[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "serde")))]
    impl<'de, K, V> Deserialize<'de> for Map<K, V>
    where
        K: Deserialize<'de> + Eq,
        V: Deserialize<'de>,
    {
        /// If deserializing a map with duplicate keys, only the first one will be kept.
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct MapVisitor<K, V> {
                marker: PhantomData<(K, V)>,
            }

            impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
            where
                K: Deserialize<'de> + Eq,
                V: Deserialize<'de>,
            {
                type Value = Map<K, V>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a map")
                }

                fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
                where
                    M: MapAccess<'de>,
                {
                    let mut map = Map::with_capacity(access.size_hint().unwrap_or(0));

                    while let Some((key, value)) = access.next_entry()? {
                        map.entry(key).or_insert(value);
                    }

                    Ok(map)
                }
            }

            deserializer.deserialize_map(MapVisitor {
                marker: PhantomData,
            })
        }
    }

    #[cfg(test)]
    mod test {
        use pretty_assertions::assert_eq;

        use super::Map;

        #[test]
        fn test_roundtrip() {
            let m = Map::from([("one fish", "two fish"), ("red fish", "blue fish")]);

            let json = serde_json::to_string(&m).unwrap();
            assert_eq!(
                json.as_str(),
                r#"{"one fish":"two fish","red fish":"blue fish"}"#
            );

            let m2: Map<&str, &str> = serde_json::from_str(&json).unwrap();
            assert_eq!(m2, m);
        }

        #[test]
        fn test_deserialize() {
            const INPUT: &str =
                r#"{"one fish":"two fish","red fish":"blue fish","red fish":"third fish"}"#;

            let m: Map<&str, &str> = serde_json::from_str(INPUT).unwrap();
            assert_eq!(
                Map::from([("one fish", "two fish"), ("red fish", "blue fish")]),
                m,
                "Duplicate keys should be deduplicated, and the first one should be kept."
            );
        }
    }
}

// taken from libstd/collections/hash/map.rs @ 7454b2
#[cfg(test)]
mod test {
    use core::cell::RefCell;

    use pretty_assertions::assert_eq;
    use rand::{thread_rng, Rng};

    use super::{
        Entry::{Occupied, Vacant},
        Map,
    };

    #[test]
    fn test_zero_capacities() {
        type M = Map<i32, i32>;

        let m = M::new();
        assert_eq!(m.capacity(), 0);

        let m = M::default();
        assert_eq!(m.capacity(), 0);

        let m = M::with_capacity(0);
        assert_eq!(m.capacity(), 0);

        let mut m = M::new();
        m.insert(1, 1);
        m.insert(2, 2);
        m.remove(&1);
        m.remove(&2);
        m.shrink_to_fit();
        assert_eq!(m.capacity(), 0);

        let mut m = M::new();
        m.reserve(0);
        assert_eq!(m.capacity(), 0);
    }

    #[test]
    fn test_create_capacity_zero() {
        let mut m = Map::with_capacity(0);

        assert!(m.insert(1, 1).is_none());

        assert!(m.contains_key(&1));
        assert!(!m.contains_key(&0));
    }

    #[test]
    fn test_insert() {
        let mut m = Map::new();
        assert_eq!(m.len(), 0);
        assert!(m.insert(1, 2).is_none());
        assert_eq!(m.len(), 1);
        assert!(m.insert(2, 4).is_none());
        assert_eq!(m.len(), 2);
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&2).unwrap(), 4);
    }

    #[test]
    fn test_clone() {
        let mut m = Map::new();
        assert_eq!(m.len(), 0);
        assert!(m.insert(1, 2).is_none());
        assert_eq!(m.len(), 1);
        assert!(m.insert(2, 4).is_none());
        assert_eq!(m.len(), 2);
        let m2 = m.clone();
        assert_eq!(*m2.get(&1).unwrap(), 2);
        assert_eq!(*m2.get(&2).unwrap(), 4);
        assert_eq!(m2.len(), 2);
    }

    thread_local! { static DROP_VECTOR: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) } }

    #[derive(PartialEq, Eq)]
    struct Droppable {
        k: usize,
    }

    impl Droppable {
        fn new(k: usize) -> Droppable {
            DROP_VECTOR.with(|slot| {
                slot.borrow_mut()[k] += 1;
            });

            Droppable { k }
        }
    }

    impl Drop for Droppable {
        fn drop(&mut self) {
            DROP_VECTOR.with(|slot| {
                slot.borrow_mut()[self.k] -= 1;
            });
        }
    }

    impl Clone for Droppable {
        fn clone(&self) -> Droppable {
            Droppable::new(self.k)
        }
    }

    #[test]
    fn test_drops() {
        DROP_VECTOR.with(|slot| {
            *slot.borrow_mut() = vec![0; 200];
        });

        {
            let mut m = Map::new();

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 0);
                }
            });

            for i in 0..100 {
                let d1 = Droppable::new(i);
                let d2 = Droppable::new(i + 100);
                m.insert(d1, d2);
            }

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 1);
                }
            });

            for i in 0..50 {
                let k = Droppable::new(i);
                let v = m.remove(&k);

                assert!(v.is_some());

                DROP_VECTOR.with(|v| {
                    assert_eq!(v.borrow()[i], 1);
                    assert_eq!(v.borrow()[i + 100], 1);
                });
            }

            DROP_VECTOR.with(|v| {
                for i in 0..50 {
                    assert_eq!(v.borrow()[i], 0);
                    assert_eq!(v.borrow()[i + 100], 0);
                }

                for i in 50..100 {
                    assert_eq!(v.borrow()[i], 1);
                    assert_eq!(v.borrow()[i + 100], 1);
                }
            });
        }

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 0);
            }
        });
    }

    #[test]
    fn test_into_iter_drops() {
        DROP_VECTOR.with(|v| {
            *v.borrow_mut() = vec![0; 200];
        });

        let hm = {
            let mut hm = Map::new();

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 0);
                }
            });

            for i in 0..100 {
                let d1 = Droppable::new(i);
                let d2 = Droppable::new(i + 100);
                hm.insert(d1, d2);
            }

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 1);
                }
            });

            hm
        };

        // By the way, ensure that cloning doesn't screw up the dropping.
        drop(hm.clone());

        {
            let mut half = hm.into_iter().take(50);

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 1);
                }
            });

            for _ in half.by_ref() {}

            DROP_VECTOR.with(|v| {
                let nk = (0..100).filter(|&i| v.borrow()[i] == 1).count();

                let nv = (0..100).filter(|&i| v.borrow()[i + 100] == 1).count();

                assert_eq!(nk, 50);
                assert_eq!(nv, 50);
            });
        };

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 0);
            }
        });
    }

    #[test]
    fn test_empty_remove() {
        let mut m: Map<i32, bool> = Map::new();
        assert_eq!(m.remove(&0), None);
    }

    #[test]
    fn test_empty_entry() {
        let mut m: Map<i32, bool> = Map::new();
        match m.entry(0) {
            Occupied(_) => panic!(),
            Vacant(_) => {}
        }
        assert!(*m.entry(0).or_insert(true));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_empty_iter() {
        let mut m: Map<i32, bool> = Map::new();
        assert_eq!(m.drain().next(), None);
        assert_eq!(m.keys().next(), None);
        assert_eq!(m.values().next(), None);
        assert_eq!(m.values_mut().next(), None);
        assert_eq!(m.iter().next(), None);
        assert_eq!(m.iter_mut().next(), None);
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());
        assert_eq!(m.into_iter().next(), None);
    }

    // takes too long for non-fast map
    // #[test]
    // fn test_lots_of_insertions() {
    //     let mut m = Map::new();
    //
    //     // Try this a few times to make sure we never screw up the map's
    //     // internal state.
    //     for _ in 0..10 {
    //         assert!(m.is_empty());
    //
    //         for i in 1..1001 {
    //             assert!(m.insert(i, i).is_none());
    //
    //             for j in 1..=i {
    //                 let r = m.get(&j);
    //                 assert_eq!(r, Some(&j));
    //             }
    //
    //             for j in i + 1..1001 {
    //                 let r = m.get(&j);
    //                 assert_eq!(r, None);
    //             }
    //         }
    //
    //         for i in 1001..2001 {
    //             assert!(!m.contains_key(&i));
    //         }
    //
    //         // remove forwards
    //         for i in 1..1001 {
    //             assert!(m.remove(&i).is_some());
    //
    //             for j in 1..=i {
    //                 assert!(!m.contains_key(&j));
    //             }
    //
    //             for j in i + 1..1001 {
    //                 assert!(m.contains_key(&j));
    //             }
    //         }
    //
    //         for i in 1..1001 {
    //             assert!(!m.contains_key(&i));
    //         }
    //
    //         for i in 1..1001 {
    //             assert!(m.insert(i, i).is_none());
    //         }
    //
    //         // remove backwards
    //         for i in (1..1001).rev() {
    //             assert!(m.remove(&i).is_some());
    //
    //             for j in i..1001 {
    //                 assert!(!m.contains_key(&j));
    //             }
    //
    //             for j in 1..i {
    //                 assert!(m.contains_key(&j));
    //             }
    //         }
    //     }
    // }

    #[test]
    fn test_find_mut() {
        let mut m = Map::new();
        assert!(m.insert(1, 12).is_none());
        assert!(m.insert(2, 8).is_none());
        assert!(m.insert(5, 14).is_none());
        let new = 100;
        match m.get_mut(&5) {
            None => panic!(),
            Some(x) => *x = new,
        }
        assert_eq!(m.get(&5), Some(&new));
    }

    #[test]
    fn test_insert_overwrite() {
        let mut m = Map::new();
        assert!(m.insert(1, 2).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert!(m.insert(1, 3).is_some());
        assert_eq!(*m.get(&1).unwrap(), 3);
    }

    #[test]
    fn test_insert_conflicts() {
        let mut m = Map::with_capacity(4);
        assert!(m.insert(1, 2).is_none());
        assert!(m.insert(5, 3).is_none());
        assert!(m.insert(9, 4).is_none());
        assert_eq!(*m.get(&9).unwrap(), 4);
        assert_eq!(*m.get(&5).unwrap(), 3);
        assert_eq!(*m.get(&1).unwrap(), 2);
    }

    #[test]
    fn test_conflict_remove() {
        let mut m = Map::with_capacity(4);
        assert!(m.insert(1, 2).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert!(m.insert(5, 3).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&5).unwrap(), 3);
        assert!(m.insert(9, 4).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&5).unwrap(), 3);
        assert_eq!(*m.get(&9).unwrap(), 4);
        assert!(m.remove(&1).is_some());
        assert_eq!(*m.get(&9).unwrap(), 4);
        assert_eq!(*m.get(&5).unwrap(), 3);
    }

    #[test]
    fn test_is_empty() {
        let mut m = Map::with_capacity(4);
        assert!(m.insert(1, 2).is_none());
        assert!(!m.is_empty());
        assert!(m.remove(&1).is_some());
        assert!(m.is_empty());
    }

    #[test]
    fn test_remove() {
        let mut m = Map::new();
        m.insert(1, 2);
        assert_eq!(m.remove(&1), Some(2));
        assert_eq!(m.remove(&1), None);
    }

    #[test]
    fn test_remove_entry() {
        let mut m = Map::new();
        m.insert(1, 2);
        assert_eq!(m.remove_entry(&1), Some((1, 2)));
        assert_eq!(m.remove(&1), None);
    }

    #[test]
    fn test_iterate() {
        let mut m = Map::with_capacity(4);
        for i in 0..32 {
            assert_eq!(None, m.insert(i, i * 2));
        }
        assert_eq!(m.len(), 32);

        let mut observed: u32 = 0;
        // Ensure that we can iterate over the owned collection,
        // and that the items are owned.
        for (k, v) in m {
            assert_eq!(v, k * 2);
            observed |= 1 << k;
        }

        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_iterate_ref() {
        let mut m = Map::with_capacity(0);
        for i in 0..32 {
            assert_eq!(None, m.insert(i, i * 2));
        }
        assert_eq!(m.len(), 32);

        let mut observed: u32 = 0;
        // Ensure that we can iterate over the borrowed collection,
        // and that the items are borrowed.
        for (&k, &v) in &m {
            assert_eq!(v, k * 2);
            observed |= 1 << k;
        }

        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_iterate_mut() {
        let mut m = Map::new();
        for i in 0..32 {
            assert_eq!(None, m.insert(i, i * 2));
        }
        assert_eq!(m.len(), 32);

        let mut observed: u32 = 0;
        // Ensure that we can iterate over the mutably borrowed collection,
        // and that the items are mutably borrowed.
        for (&mut k, &mut v) in &mut m {
            assert_eq!(v, k * 2);
            observed |= 1 << k;
        }

        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_keys() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: Map<_, _> = vec.into_iter().collect();
        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }

    #[test]
    fn test_values() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: Map<_, _> = vec.into_iter().collect();
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&'a'));
        assert!(values.contains(&'b'));
        assert!(values.contains(&'c'));
    }

    #[test]
    fn test_values_mut() {
        let vec = vec![(1, 1), (2, 2), (3, 3)];
        let mut map: Map<_, _> = vec.into_iter().collect();
        for value in map.values_mut() {
            *value *= 2
        }
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&2));
        assert!(values.contains(&4));
        assert!(values.contains(&6));
    }

    #[test]
    fn test_find() {
        let mut m = Map::new();
        assert!(m.get(&1).is_none());
        m.insert(1, 2);
        match m.get(&1) {
            None => panic!(),
            Some(v) => assert_eq!(*v, 2),
        }
    }

    #[test]
    fn test_eq() {
        let mut m1 = Map::new();
        m1.insert(1, 2);
        m1.insert(2, 3);
        m1.insert(3, 4);

        let mut m2 = Map::new();
        m2.insert(1, 2);
        m2.insert(2, 3);

        assert!(m1 != m2);

        m2.insert(3, 4);

        assert_eq!(m1, m2);
    }

    #[test]
    fn test_show() {
        let mut map = Map::new();
        let empty: Map<i32, i32> = Map::new();

        map.insert(1, 2);
        map.insert(3, 4);

        let map_str = format!("{:?}", map);

        assert!(map_str == "{1: 2, 3: 4}" || map_str == "{3: 4, 1: 2}");
        assert_eq!(format!("{:?}", empty), "{}");
    }

    #[test]
    fn test_reserve_shrink_to_fit() {
        let mut m = Map::new();
        m.insert(0, 0);
        m.remove(&0);
        assert!(m.capacity() >= m.len());
        for i in 0..128 {
            m.insert(i, i);
        }
        m.reserve(256);

        let usable_cap = m.capacity();
        for i in 128..(128 + 256) {
            m.insert(i, i);
            assert_eq!(m.capacity(), usable_cap);
        }

        for i in 100..(128 + 256) {
            assert_eq!(m.remove(&i), Some(i));
        }
        m.shrink_to_fit();

        assert_eq!(m.len(), 100);
        assert!(!m.is_empty());
        assert!(m.capacity() >= m.len());

        for i in 0..100 {
            assert_eq!(m.remove(&i), Some(i));
        }
        m.shrink_to_fit();
        m.insert(0, 0);

        assert_eq!(m.len(), 1);
        assert!(m.capacity() >= m.len());
        assert_eq!(m.remove(&0), Some(0));
    }

    #[test]
    fn test_from_iter() {
        let xs = [
            (1_i32, 1_i32),
            (2, -1), // This one will be put into the map first
            (2, 2),  // But this one will clobber the previous one
            (3, 3),
            (4, 4),
            (5, 5),
            (6, 6),
        ];

        let map: Map<_, _> = xs.iter().cloned().collect();

        for &(k, v) in xs.iter().filter(|(_k, v)| v.is_positive()) {
            assert_eq!(map.get(&k), Some(&v));
        }

        // -1 because the `2` key is duplicated in the source array, and the
        // collected `Map` will contain unique keys.
        assert_eq!(map.iter().len(), xs.len() - 1);
        assert_eq!(map.len(), xs.len() - 1);
        assert_eq!(map.capacity(), xs.len() - 1);
    }

    #[test]
    fn test_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: Map<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_iter_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: Map<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.len(), 3);
    }

    #[test]
    fn test_double_ended() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: Map<_, _> = xs.iter().cloned().collect();

        let last = map.iter().next_back();

        assert_eq!(last, xs.last().map(|item| (&item.0, &item.1)));
    }

    #[test]
    fn test_mut_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: Map<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter_mut();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_iter_mut_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: Map<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter_mut();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.len(), 3);
    }

    #[test]
    fn test_mut_double_ended() {
        let mut xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: Map<_, _> = xs.iter().cloned().collect();

        let last = map.iter_mut().next_back();

        assert_eq!(last, xs.last_mut().map(|item| (&mut item.0, &mut item.1)));
    }

    #[test]
    fn test_index() {
        let mut map = Map::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        assert_eq!(map[&2], 1);
    }

    #[test]
    #[should_panic]
    fn test_index_nonexistent() {
        let mut map = Map::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        _ = map[&4];
    }

    #[test]
    fn test_entry() {
        let xs = [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)];

        let mut map: Map<_, _> = xs.iter().cloned().collect();

        // Existing key (insert)
        match map.entry(1) {
            Vacant(_) => unreachable!(),
            Occupied(mut view) => {
                assert_eq!(view.get(), &10);
                assert_eq!(view.insert(100), 10);
            }
        }
        assert_eq!(map.get(&1).unwrap(), &100);
        assert_eq!(map.len(), 6);

        // Existing key (update)
        match map.entry(2) {
            Vacant(_) => unreachable!(),
            Occupied(mut view) => {
                let v = view.get_mut();
                let new_v = (*v) * 10;
                *v = new_v;
            }
        }
        assert_eq!(map.get(&2).unwrap(), &200);
        assert_eq!(map.len(), 6);

        // Existing key (take)
        match map.entry(3) {
            Vacant(_) => unreachable!(),
            Occupied(view) => {
                assert_eq!(view.remove(), 30);
            }
        }
        assert_eq!(map.get(&3), None);
        assert_eq!(map.len(), 5);

        // Inexistent key (insert)
        match map.entry(10) {
            Occupied(_) => unreachable!(),
            Vacant(view) => {
                assert_eq!(*view.insert(1000), 1000);
            }
        }
        assert_eq!(map.get(&10).unwrap(), &1000);
        assert_eq!(map.len(), 6);
    }

    #[test]
    fn test_entry_take_doesnt_corrupt() {
        // Test for #19292
        fn check(m: &Map<i32, ()>) {
            for k in m.keys() {
                assert!(m.contains_key(k), "{} is in keys() but not in the map?", k);
            }
        }

        let mut m = Map::new();
        let mut rng = thread_rng();

        // Populate the map with some items.
        for _ in 0..50 {
            let x = rng.gen_range(-10..10);
            m.insert(x, ());
        }

        for _ in 0..1000 {
            let x = rng.gen_range(-10..10);
            match m.entry(x) {
                Vacant(_) => {}
                Occupied(e) => {
                    e.remove();
                }
            }

            check(&m);
        }
    }

    #[test]
    fn test_extend_ref() {
        let mut a = Map::new();
        a.insert(1, "one");
        let mut b = Map::new();
        b.insert(2, "two");
        b.insert(3, "three");

        a.extend(&b);

        assert_eq!(a.len(), 3);
        assert_eq!(a[&1], "one");
        assert_eq!(a[&2], "two");
        assert_eq!(a[&3], "three");
    }

    #[test]
    fn test_capacity_not_less_than_len() {
        let mut a = Map::new();
        let mut item = 0;

        for _ in 0..116 {
            a.insert(item, 0);
            item += 1;
        }

        assert!(a.capacity() > a.len());

        let free = a.capacity() - a.len();
        for _ in 0..free {
            a.insert(item, 0);
            item += 1;
        }

        assert_eq!(a.len(), a.capacity());

        // Insert at capacity should cause allocation.
        a.insert(item, 0);
        assert!(a.capacity() > a.len());
    }

    #[test]
    fn test_occupied_entry_key() {
        let mut a = Map::new();
        let key = "hello there";
        let value = "value goes here";
        assert!(a.is_empty());
        a.insert(key, value);
        assert_eq!(a.len(), 1);
        assert_eq!(a[key], value);

        match a.entry(key) {
            Vacant(_) => panic!(),
            Occupied(e) => assert_eq!(key, *e.key()),
        }
        assert_eq!(a.len(), 1);
        assert_eq!(a[key], value);
    }

    #[test]
    fn test_vacant_entry_key() {
        let mut a = Map::new();
        let key = "hello there";
        let value = "value goes here";

        assert!(a.is_empty());
        match a.entry(key) {
            Occupied(_) => panic!(),
            Vacant(e) => {
                assert_eq!(key, *e.key());
                e.insert(value);
            }
        }
        assert_eq!(a.len(), 1);
        assert_eq!(a[key], value);
    }

    #[test]
    fn test_retain() {
        let mut map: Map<i32, i32> = (0..100).map(|x| (x, x * 10)).collect();

        map.retain(|&k, _| k % 2 == 0);
        assert_eq!(map.len(), 50);
        assert_eq!(map[&2], 20);
        assert_eq!(map[&4], 40);
        assert_eq!(map[&6], 60);
    }

    #[test]
    fn test_try_reserve() {
        let mut empty_bytes: Map<u8, u8> = Map::new();

        const MAX_USIZE: usize = usize::MAX;

        let _err = empty_bytes
            .try_reserve(MAX_USIZE)
            .expect_err("usize::MAX should trigger an overflow!");
        #[cfg(feature = "nightly")]
        {
            assert_eq!(
                alloc::collections::TryReserveErrorKind::CapacityOverflow,
                _err.kind()
            );
        }

        let _err = empty_bytes
            .try_reserve(MAX_USIZE / 8)
            .expect_err("usize::MAX / 8 should trigger an OOM!");
        #[cfg(feature = "nightly")]
        {
            assert!(matches!(
                _err.kind(),
                alloc::collections::TryReserveErrorKind::AllocError { .. },
            ));
        }
    }

    #[test]
    fn test_debug_format() {
        let mut a = Map::<&str, usize>::default();
        assert_eq!("{}", format!("{:?}", a));

        a.insert("a", 1);
        assert_eq!(r#"{"a": 1}"#, format!("{:?}", a));

        a.insert("b", 2);
        assert_eq!(r#"{"a": 1, "b": 2}"#, format!("{:?}", a));
    }

    /// Ensures that, like `Vec`, `Default` works for `Map` even when its
    /// key/value types do not implement `Default`.
    #[test]
    fn test_default() {
        struct NoDefault;

        let _: Vec<NoDefault> = Default::default();
        let _: Map<NoDefault, NoDefault> = Default::default();
    }

    /// Ensures that things that can be turned `Into` a `Vec<(K, V)>` can also be turned into a `Map<K, V>`
    #[test]
    fn test_from_into_vec() {
        #[allow(
            clippy::useless_conversion,
            // reason = "Being consistent about the desired type"
        )]
        let _: Vec<(char, u32)> = vec![('a', 1)].into();
        let _: Map<char, u32> = vec![('a', 1)].into();
        let _: Vec<(char, u32)> = [('a', 1)].into();
        let _: Map<char, u32> = [('a', 1)].into();

        let expected: Map<char, u32> = [('a', 3), ('b', 2)].iter().copied().collect();
        let actual: Map<char, u32> = [('a', 1), ('b', 2), ('a', 3)].into();
        assert_eq!(expected, actual, "Keys should be de-duped");
    }
}
