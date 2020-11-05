use core::cmp::Ordering;

pub mod iter;
pub mod display;

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

/* A simple key-value, that isn't ordered */

pub struct KeyValue<K, V> {
    pub key:   K,
    pub value: V
}

/* Stacks should be implemented using Vec */

pub type Stack<T> = Vec<T>;

/* Either */

#[allow(dead_code)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
}

/* Option utils */

pub fn some_if<T>(cond: bool, value: T) -> Option<T> {
    if cond { Some(value) }
    else { None }
}
