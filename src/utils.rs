use core::cmp::Ordering;

pub mod iter;
pub mod display;
pub mod fairheap;

/* A wrapper around KeyValue, ordered by key */

#[derive(Clone)]
pub struct OrdByKey<K, V>(pub KeyValue<K, V>);

impl<K, V> PartialEq for OrdByKey<K, V>
    where K: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.0.key.eq(&other.0.key)
    }
}

impl<K, V> Eq for OrdByKey<K, V>
    where K: Eq
    { }

impl<K, V> PartialOrd for OrdByKey<K, V>
    where K: PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.key.partial_cmp(&other.0.key)
    }
}

impl<K, V> Ord for OrdByKey<K, V>
    where K: Ord
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.key.cmp(&other.0.key)
    }
}

impl<K, V> OrdByKey<K, V> {
    pub fn new(key: K, value: V) -> Self {
        OrdByKey (
            KeyValue {
                key,
                value
            }
        )
    }
}

/* A simple key-value, that isn't ordered */

#[derive(Clone)]
pub struct KeyValue<K, V> {
    pub key:   K,
    pub value: V
}

/* Stacks should be implemented using Vec */

pub type Stack<T> = Vec<T>;

/* Option utils */

pub fn some_if<T>(cond: bool, value: T) -> Option<T> {
    if cond { Some(value) }
    else { None }
}

/* Either */

#[derive(Clone, Copy)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
}
use Either::*;

impl<L, R> Either<L, R> {

    pub fn unwrap_left(self) -> L {
        match self {
            Left(elem) => elem,
            _          => panic!("Expected Left, found Right"),
        }
    }

    pub fn unwrap_right(self) -> R {
        match self {
            Right(elem) => elem,
            _           => panic!("Expected Right, found Left"),
        }
    }

}
