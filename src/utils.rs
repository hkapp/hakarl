use core::cmp::Ordering;

pub mod iter;

/* Simple key-value type, ordered by key */

pub struct OrdBy<K, V> {
    pub ord_key: K,
    pub data:    V
}

impl<K, V> PartialEq for OrdBy<K, V>
    where K: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.ord_key.eq(&other.ord_key)
    }
}

impl<K, V> Eq for OrdBy<K, V>
    where K: PartialEq { }

impl<K, V> PartialOrd for OrdBy<K, V>
    where K: PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ord_key.partial_cmp(&other.ord_key)
    }
}

impl<K, V> Ord for OrdBy<K, V>
    where K: Ord
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.ord_key.cmp(&other.ord_key)
    }
}

/* Either */

#[allow(dead_code)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
}
