use alloc::vec::Vec;
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    iter::FusedIterator,
    slice::Iter,
};

/// `Set` is a data structure with a [`HashSet`]-like API but based on a `Vec`.
///
/// It's primarily useful when you care about constant factors or prefer determinism to speed.
/// Please refer to the docs for [`HashSet`] for details and examples of the Set API.
///
/// ## Example
///
/// ```
/// let mut set1 = map_vec::Set::new();
/// let mut set2 = map_vec::Set::new();
/// set1.insert(1);
/// set1.insert(2);
/// set2.insert(2);
/// set2.insert(3);
/// let mut set3 = map_vec::Set::with_capacity(1);
/// assert!(set3.insert(3));
/// assert_eq!(&set2 - &set1, set3);
/// ```
///
/// [`HashSet`]: std::collections::HashSet
#[derive(Clone, PartialEq, Eq)]
pub struct Set<T> {
    backing: Vec<T>,
}

impl<T> Default for Set<T> {
    fn default() -> Self {
        Self {
            backing: Vec::default(),
        }
    }
}

impl<T: Eq> Set<T> {
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

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.backing.iter().any(|v| value.eq(v.borrow()))
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, T> {
        Difference {
            iter: self.iter(),
            other,
        }
    }

    pub fn drain(&mut self) -> alloc::vec::Drain<T> {
        self.backing.drain(..)
    }

    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.backing.iter().find(|v| value.eq((*v).borrow()))
    }

    pub fn get_or_insert(&mut self, value: T) -> &T {
        // TODO: One day, rustc will be smart enough for this.
        // Needs Polonius to complete the non-lexical lifetimes (NLL).
        // https://blog.rust-lang.org/2022/08/05/nll-by-default.html
        //
        // match self.get(&value) {
        //     Some(existing) => existing,
        //     None => {
        //         self.backing.push(value);
        //         self.backing.last().unwrap()
        //     }
        // }

        let self_ptr = self as *mut Self;

        if let Some(value) = self.get(&value) {
            return value;
        }

        // SAFETY: self_ptr is not null and is not otherwise borrowed.
        // This is needed until the NLL-related solution above works in stable Rust.
        unsafe { (*self_ptr).backing.push(value) };

        self.backing.last().unwrap()
    }

    pub fn get_or_insert_with<Q>(&mut self, value: &Q, f: impl FnOnce(&Q) -> T) -> &T
    where
        T: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        // TODO: One day, rustc will be smart enough for this.
        //       https://stackoverflow.com/a/38031183/297468
        // self.get(&value).unwrap_or_else(|| {
        //     self.backing.push(f(value));
        //     self.backing.last().unwrap()
        // })

        let self_ptr = self as *mut Self;

        if let Some(value) = self.get(value) {
            return value;
        }

        // SAFETY: self_ptr is not null and is not otherwise borrowed.
        // This is needed until the NLL-related solution above works in stable Rust.
        unsafe { (*self_ptr).backing.push(f(value)) };

        self.backing.last().unwrap()
    }

    pub fn insert(&mut self, value: T) -> bool {
        !self.backing.iter().any(|v| *v == value) && {
            self.backing.push(value);
            true
        }
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, T> {
        Intersection {
            iter: self.iter(),
            other,
        }
    }

    pub fn is_disjoint<'a>(&'a self, other: &'a Self) -> bool {
        self.intersection(other).count() == 0
    }

    pub fn is_empty(&self) -> bool {
        self.backing.is_empty()
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.len() <= other.len() && self.difference(other).count() == 0
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    pub fn iter(&self) -> Iter<T> {
        self.backing.iter()
    }

    pub fn len(&self) -> usize {
        self.backing.len()
    }

    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.take(value).is_some()
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        match self.backing.iter_mut().find(|v| **v == value) {
            Some(v) => Some(core::mem::replace(v, value)),
            None => {
                self.backing.push(value);
                None
            }
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.backing.reserve(additional)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.backing.retain(f);
    }

    pub fn shrink_to_fit(&mut self) {
        self.backing.shrink_to_fit()
    }

    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifference<'a, T> {
        SymmetricDifference {
            iter: self.difference(other).chain(other.difference(self)),
        }
    }

    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.backing
            .iter()
            .position(|v| value.eq(v.borrow()))
            .map(|pos| self.backing.swap_remove(pos))
    }

    pub fn union<'a>(&'a self, other: &'a Self) -> Union<'a, T> {
        Union {
            iter: self.iter().chain(other.difference(self)),
        }
    }

    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), alloc::collections::TryReserveError> {
        self.backing.try_reserve(additional)
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.backing.shrink_to(min_capacity)
    }
}

impl<T: Debug> fmt::Debug for Set<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.backing.iter()).finish()
    }
}

impl<'a, T> IntoIterator for &'a Set<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.backing.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Set<T> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.backing.iter_mut()
    }
}

impl<T> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.backing.into_iter()
    }
}

impl<T: Eq> FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
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

impl<T: Eq> Extend<T> for Set<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<'a, T: 'a + Copy + Eq> Extend<&'a T> for Set<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(*item);
        }
    }
}

impl<V: Eq, T: Into<Vec<V>>> From<T> for Set<V> {
    fn from(values: T) -> Self {
        let values = values.into();
        let mut map = Self::with_capacity(values.len());
        map.extend(values);
        map.shrink_to_fit();
        map
    }
}

impl<T: Clone + Eq> core::ops::BitOr<&Set<T>> for &Set<T> {
    type Output = Set<T>;
    fn bitor(self, rhs: &Set<T>) -> Set<T> {
        self.union(rhs).cloned().collect()
    }
}

impl<T: Clone + Eq> core::ops::BitAnd<&Set<T>> for &Set<T> {
    type Output = Set<T>;
    fn bitand(self, rhs: &Set<T>) -> Set<T> {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T: Clone + Eq> core::ops::BitXor<&Set<T>> for &Set<T> {
    type Output = Set<T>;
    fn bitxor(self, rhs: &Set<T>) -> Set<T> {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T: Clone + Eq> core::ops::Sub<&Set<T>> for &Set<T> {
    type Output = Set<T>;
    fn sub(self, rhs: &Set<T>) -> Set<T> {
        self.difference(rhs).cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct Difference<'a, T> {
    iter: core::slice::Iter<'a, T>,
    other: &'a Set<T>,
}

impl<'a, T> Iterator for Difference<'a, T>
where
    T: Eq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let elt = self.iter.next()?;
            if !self.other.contains(elt) {
                return Some(elt);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<T> DoubleEndedIterator for Difference<'_, T>
where
    T: Eq,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let elt = self.iter.next_back()?;
            if !self.other.contains(elt) {
                return Some(elt);
            }
        }
    }
}

impl<T> FusedIterator for Difference<'_, T> where T: Eq {}

#[derive(Debug, Clone)]
pub struct Intersection<'a, T> {
    iter: core::slice::Iter<'a, T>,
    other: &'a Set<T>,
}

impl<'a, T> Iterator for Intersection<'a, T>
where
    T: Eq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let elt = self.iter.next()?;
            if self.other.contains(elt) {
                return Some(elt);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<T> DoubleEndedIterator for Intersection<'_, T>
where
    T: Eq,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let elt = self.iter.next_back()?;
            if self.other.contains(elt) {
                return Some(elt);
            }
        }
    }
}

impl<T> FusedIterator for Intersection<'_, T> where T: Eq {}

#[derive(Debug, Clone)]
pub struct SymmetricDifference<'a, T> {
    iter: core::iter::Chain<Difference<'a, T>, Difference<'a, T>>,
}

impl<'a, T> Iterator for SymmetricDifference<'a, T>
where
    T: Eq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<T> DoubleEndedIterator for SymmetricDifference<'_, T>
where
    T: Eq,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<T> FusedIterator for SymmetricDifference<'_, T> where T: Eq {}

#[derive(Debug, Clone)]
pub struct Union<'a, T> {
    iter: core::iter::Chain<Iter<'a, T>, Difference<'a, T>>,
}

impl<'a, T> Iterator for Union<'a, T>
where
    T: Eq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<T> DoubleEndedIterator for Union<'_, T>
where
    T: Eq,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<T> FusedIterator for Union<'_, T> where T: Eq {}

#[cfg(feature = "serde")]
mod set_serde {
    use core::{fmt, marker::PhantomData};

    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeSeq,
        Deserialize, Deserializer, Serialize, Serializer,
    };

    use super::Set;

    #[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "serde")))]
    impl<T> Serialize for Set<T>
    where
        T: Serialize + Eq,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.len()))?;
            for item in self {
                seq.serialize_element(item)?;
            }
            seq.end()
        }
    }

    #[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "serde")))]
    impl<'de, T> Deserialize<'de> for Set<T>
    where
        T: Deserialize<'de> + Eq,
    {
        /// If deserializing a sequence with duplicate values, only the first one will be kept.
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct SetVisitor<T> {
                marker: PhantomData<T>,
            }

            impl<'de, T> Visitor<'de> for SetVisitor<T>
            where
                T: Deserialize<'de> + Eq,
            {
                type Value = Set<T>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                where
                    S: SeqAccess<'de>,
                {
                    let mut set = Set::with_capacity(seq.size_hint().unwrap_or(0));

                    while let Some(item) = seq.next_element()? {
                        set.get_or_insert(item);
                    }

                    Ok(set)
                }
            }

            deserializer.deserialize_seq(SetVisitor {
                marker: PhantomData,
            })
        }
    }

    #[cfg(test)]
    mod test {
        use pretty_assertions::assert_eq;

        use super::Set;

        #[test]
        fn test_roundtrip() {
            let s = Set::from(["one fish", "two fish", "red fish", "blue fish"]);

            let json = serde_json::to_string(&s).unwrap();
            assert_eq!(
                json.as_str(),
                r#"["one fish","two fish","red fish","blue fish"]"#
            );

            let s2: Set<&str> = serde_json::from_str(&json).unwrap();
            assert_eq!(s2, s);
        }

        #[test]
        fn test_deserialize() {
            const INPUT: &str =
                r#"["one fish","two fish","red fish","blue fish","red fish","third fish"]"#;

            let m: Set<&str> = serde_json::from_str(INPUT).unwrap();
            assert_eq!(
                Set::from([
                    "one fish",
                    "two fish",
                    "red fish",
                    "blue fish",
                    "third fish"
                ]),
                m,
                "Duplicate keys should be deduplicated, and the first one should be kept."
            );
        }
    }
}

// taken from libstd/collections/hash/set.rs @ 7454b2
#[cfg(test)]
mod test_set {
    use pretty_assertions::assert_eq;

    use super::Set;

    #[test]
    fn test_zero_capacities() {
        type S = Set<i32>;

        let s = S::new();
        assert_eq!(s.capacity(), 0);

        let s = S::default();
        assert_eq!(s.capacity(), 0);

        let s = S::with_capacity(0);
        assert_eq!(s.capacity(), 0);

        let mut s = S::new();
        s.insert(1);
        s.insert(2);
        s.remove(&1);
        s.remove(&2);
        s.shrink_to_fit();
        assert_eq!(s.capacity(), 0);

        let mut s = S::new();
        s.reserve(0);
        assert_eq!(s.capacity(), 0);
    }

    #[test]
    fn test_disjoint() {
        let mut xs = Set::new();
        let mut ys = Set::new();
        assert!(xs.is_disjoint(&ys));
        assert!(ys.is_disjoint(&xs));
        assert!(xs.insert(5));
        assert!(ys.insert(11));
        assert!(xs.is_disjoint(&ys));
        assert!(ys.is_disjoint(&xs));
        assert!(xs.insert(7));
        assert!(xs.insert(19));
        assert!(xs.insert(4));
        assert!(ys.insert(2));
        assert!(ys.insert(-11));
        assert!(xs.is_disjoint(&ys));
        assert!(ys.is_disjoint(&xs));
        assert!(ys.insert(7));
        assert!(!xs.is_disjoint(&ys));
        assert!(!ys.is_disjoint(&xs));
    }

    #[test]
    fn test_subset_and_superset() {
        let mut a = Set::new();
        assert!(a.insert(0));
        assert!(a.insert(5));
        assert!(a.insert(11));
        assert!(a.insert(7));

        let mut b = Set::new();
        assert!(b.insert(0));
        assert!(b.insert(7));
        assert!(b.insert(19));
        assert!(b.insert(250));
        assert!(b.insert(11));
        assert!(b.insert(200));

        assert!(!a.is_subset(&b));
        assert!(!a.is_superset(&b));
        assert!(!b.is_subset(&a));
        assert!(!b.is_superset(&a));

        assert!(b.insert(5));

        assert!(a.is_subset(&b));
        assert!(!a.is_superset(&b));
        assert!(!b.is_subset(&a));
        assert!(b.is_superset(&a));
    }

    #[test]
    fn test_iterate() {
        let mut a = Set::new();
        for i in 0..32 {
            assert!(a.insert(i));
        }
        assert_eq!(a.len(), 32);

        let mut observed: u32 = 0;
        // Ensure that we can iterate over the owned collection,
        // and that the items are owned.
        for k in a {
            observed |= 1 << k;
        }

        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_iterate_ref() {
        let a = Set::from_iter(0..32);
        assert_eq!(a.len(), 32);

        let mut observed: u32 = 0;
        // Ensure that we can iterate over the borrowed collection,
        // and that the items are borrowed.
        for &k in &a {
            observed |= 1 << k;
        }

        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_iterate_mut() {
        let mut a: Set<_> = (0..32).collect();
        assert_eq!(a.len(), 32);

        let mut observed: u32 = 0;
        // Ensure that we can iterate over the mutably borrowed collection,
        // and that the items are mutably borrowed.
        for &mut k in &mut a {
            observed |= 1 << k;
        }

        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_intersection() {
        let mut a = Set::new();
        let mut b = Set::new();
        assert!(a.intersection(&b).next().is_none());

        assert!(a.insert(11));
        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(77));
        assert!(a.insert(103));
        assert!(a.insert(5));
        assert!(a.insert(-5));

        assert!(b.insert(2));
        assert!(b.insert(11));
        assert!(b.insert(77));
        assert!(b.insert(-9));
        assert!(b.insert(-42));
        assert!(b.insert(5));
        assert!(b.insert(3));

        let mut i = 0;
        let expected = [3, 5, 11, 77];
        for x in a.intersection(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());

        assert!(a.insert(9)); // make a bigger than b

        i = 0;
        for x in a.intersection(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());

        i = 0;
        for x in b.intersection(&a) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_difference() {
        let mut a = Set::new();
        let mut b = Set::new();

        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(5));
        assert!(a.insert(9));
        assert!(a.insert(11));

        assert!(b.insert(3));
        assert!(b.insert(9));

        let mut i = 0;
        let expected = [1, 5, 11];
        for x in a.difference(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_symmetric_difference() {
        let mut a = Set::new();
        let mut b = Set::new();

        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(5));
        assert!(a.insert(9));
        assert!(a.insert(11));

        assert!(b.insert(-2));
        assert!(b.insert(3));
        assert!(b.insert(9));
        assert!(b.insert(14));
        assert!(b.insert(22));

        let mut i = 0;
        let expected = [-2, 1, 5, 11, 14, 22];
        for x in a.symmetric_difference(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_union() {
        let mut a = Set::new();
        let mut b = Set::new();
        assert!(a.union(&b).next().is_none());
        assert!(b.union(&a).next().is_none());

        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(11));
        assert!(a.insert(16));
        assert!(a.insert(19));
        assert!(a.insert(24));

        assert!(b.insert(-2));
        assert!(b.insert(1));
        assert!(b.insert(5));
        assert!(b.insert(9));
        assert!(b.insert(13));
        assert!(b.insert(19));

        let mut i = 0;
        let expected = [-2, 1, 3, 5, 9, 11, 13, 16, 19, 24];
        for x in a.union(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());

        assert!(a.insert(9)); // make a bigger than b
        assert!(a.insert(5));

        i = 0;
        for x in a.union(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());

        i = 0;
        for x in b.union(&a) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_from_iter() {
        let xs = [1, 2, 2, 3, 4, 5, 6, 7, 8, 9];

        let set: Set<_> = xs.iter().cloned().collect();

        for x in &xs {
            assert!(set.contains(x));
        }

        // -1 because `2` is duplicated in the source array, and the collected
        // `Set` will contain unique values.
        assert_eq!(set.iter().len(), xs.len() - 1);
    }

    #[test]
    fn test_move_iter() {
        let hs = {
            let mut hs = Set::new();

            hs.insert('a');
            hs.insert('b');

            hs
        };

        let v = hs.into_iter().collect::<Vec<char>>();
        assert!(v == ['a', 'b'] || v == ['b', 'a']);
    }

    #[test]
    fn test_eq() {
        // These constants once happened to expose a bug in insert().
        // I'm keeping them around to prevent a regression.
        let mut s1 = Set::new();

        s1.insert(1);
        s1.insert(2);
        s1.insert(3);

        let mut s2 = Set::new();

        s2.insert(1);
        s2.insert(2);

        assert!(s1 != s2);

        s2.insert(3);

        assert_eq!(s1, s2);
    }

    #[test]
    fn test_show() {
        let mut set = Set::new();
        let empty = Set::<i32>::new();

        set.insert(1);
        set.insert(2);

        let set_str = format!("{:?}", set);

        assert!(set_str == "{1, 2}" || set_str == "{2, 1}");
        assert_eq!(format!("{:?}", empty), "{}");
    }

    #[test]
    fn test_trivial_drain() {
        let mut s = Set::<i32>::new();
        for _ in s.drain() {}
        assert!(s.is_empty());
        drop(s);

        let mut s = Set::<i32>::new();
        drop(s.drain());
        assert!(s.is_empty());
    }

    #[test]
    fn test_drain() {
        let mut s: Set<_> = (1..100).collect();

        // try this a bunch of times to make sure we don't screw up internal state.
        for _ in 0..20 {
            assert_eq!(s.len(), 99);

            {
                let mut last_i = 0;
                let mut d = s.drain();
                for (i, x) in d.by_ref().take(50).enumerate() {
                    last_i = i;
                    assert!(x != 0);
                }
                assert_eq!(last_i, 49);
            }

            assert_eq!(s.iter().next(), None, "s should be empty!");

            // reset to try again.
            s.extend(1..100);
        }
    }

    #[test]
    fn test_replace() {
        #[derive(Debug)]
        struct Foo(&'static str, i32);

        impl PartialEq for Foo {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for Foo {}

        let mut s = Set::new();
        assert_eq!(s.replace(Foo("a", 1)), None);
        assert_eq!(s.len(), 1);

        let existing = s
            .replace(Foo("a", 2))
            .expect("Did not get the existing item");

        // Do _not_ use `assert_eq!(Foo("a", 1), existing)` here as the `PartialEq`
        // implementation only checks `Foo.0`, but we also need to check `Foo.1`.
        let Foo(a, b) = existing;
        assert_eq!(a, "a");
        assert_eq!(b, 1);

        assert_eq!(s.len(), 1);

        let mut it = s.iter();

        // Do _not_ use `assert_eq!()` here as the `PartialEq` implementation
        // only checks `Foo.0`, but we also need to check `Foo.1`.
        let item = it.next().expect("Should get an item from the iterator");
        let &Foo(a, b) = item;
        assert_eq!(a, "a");
        assert_eq!(b, 2);

        assert_eq!(it.next(), None, "Should be no more items in the iterator");
    }

    #[test]
    fn test_extend_ref() {
        let mut a = Set::new();
        a.insert(1);

        a.extend(&[2, 3, 4]);

        assert_eq!(a.len(), 4);
        assert!(a.contains(&1));
        assert!(a.contains(&2));
        assert!(a.contains(&3));
        assert!(a.contains(&4));

        let mut b = Set::new();
        b.insert(5);
        b.insert(6);

        a.extend(&b);

        assert_eq!(a.len(), 6);
        assert!(a.contains(&1));
        assert!(a.contains(&2));
        assert!(a.contains(&3));
        assert!(a.contains(&4));
        assert!(a.contains(&5));
        assert!(a.contains(&6));
    }

    #[test]
    fn test_retain() {
        let xs = [1, 2, 3, 4, 5, 6];
        let mut set: Set<i32> = xs.iter().cloned().collect();
        set.retain(|&k| k % 2 == 0);
        assert_eq!(set.len(), 3);
        assert!(set.contains(&2));
        assert!(set.contains(&4));
        assert!(set.contains(&6));
    }

    /// Ensures that, like `Vec`, `Default` works for `Set` even when its value
    /// type does not implement `Default`.
    #[test]
    fn test_default() {
        struct NoDefault;

        let _: Vec<NoDefault> = Default::default();
        let _: Set<NoDefault> = Default::default();
    }

    /// Ensures that things that can be turned `Into` a `Vec` can also be turned into a `Set`
    #[test]
    fn test_from_into_vec() {
        #[allow(
            clippy::useless_conversion,
            // reason = "Being consistent about the desired type"
        )]
        let _: Vec<()> = vec![()].into();
        let _: Set<()> = vec![()].into();
        let _: Vec<()> = [()].into();
        let _: Set<()> = [()].into();

        let expected: Set<char> = ['a', 'b'].iter().copied().collect();
        let actual: Set<char> = ['a', 'b', 'a'].into();
        assert_eq!(expected, actual, "Values should be de-duped");
    }
}
