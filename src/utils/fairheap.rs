use core::cmp::Ordering;
use std::collections::BinaryHeap;
use rand;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;

type SomeRng = rand::rngs::StdRng;

#[derive(Clone)]
pub struct FairHeap<T> {
    eq_best:    Vec<T>,
    all_others: BinaryHeap<T>,
    rng:        RefCell<SomeRng>  /* interior mutable */
}

impl<T: Ord> FairHeap<T> {
    pub fn new() -> FairHeap<T> {
        let rng = SomeRng::from_entropy();

        FairHeap {
            eq_best:    Vec::new(),
            all_others: BinaryHeap::new(),
            rng:        RefCell::new(rng)
        }
    }

    pub fn push(&mut self, item: T) {
        if self.eq_best.is_empty() {
            /* Initialization: this DS is empty */
            self.eq_best.push(item);
        }
        else {
            /* Compare the new value to the best ones we know */
            match item.cmp(&self.eq_best[0]) {
                Ordering::Less    => self.all_others.push(item),  /* Worse than the best */
                Ordering::Equal   => self.eq_best.push(item),     /* Equivalent to the best ones */
                Ordering::Greater => {                            /* Better than the best ones */
                    self.demote();
                    self.eq_best.push(item);
                }
            }
        }
    }

    fn demote(&mut self) {
        for prev_best in self.eq_best.drain(..) {
            self.all_others.push(prev_best);
        }
    }

    fn fair_pop_index(&self) -> Option<usize> {
        if self.eq_best.is_empty() {
            None
        }
        else {
            let fair_idx = self.rng.borrow_mut().gen_range(0, self.eq_best.len());
            Some(fair_idx)
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.eq_best.is_empty() {
            return None;
        }

        let fair_idx = self.fair_pop_index().unwrap();
        let res = self.eq_best.remove(fair_idx);

        if self.eq_best.is_empty() {
            self.promote();
        }

        return Some(res);
    }

    fn promote(&mut self) {
        if self.all_others.is_empty() {
            return;
        }

        let new_best = self.all_others.pop().unwrap();
        self.eq_best.push(new_best);

        let heap = &mut self.all_others;
        while let Some(next_best) = heap.pop() {
            match next_best.cmp(&self.eq_best[0]) {
                Ordering::Greater => panic!("BinaryHeap invariant broken"),
                Ordering::Equal   => self.eq_best.push(next_best),
                Ordering::Less    => {
                    /* reached the end of the equivalence class */
                    heap.push(next_best);  /* push the item back */
                    return;
                }
            }
        }
    }

    pub fn into_sorted_vec(self) -> Vec<T> {
        let mut res = self.eq_best;
        let mut heap_res = self.all_others.into_sorted_vec();
        res.append(&mut heap_res);
        return res;
    }

    pub fn peek(&self) -> Option<&T> {
        self.fair_pop_index()
            .map(|fair_idx| &self.eq_best[fair_idx])
    }

    pub fn is_empty(&self) -> bool {
        self.eq_best.is_empty()
    }
}

impl<T: Ord> Default for FairHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}
