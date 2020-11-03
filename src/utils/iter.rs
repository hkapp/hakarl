
/* Get all the max values */

pub fn all_maxs_by_key<B, F, I>(mut iter: I, mut f: F) -> Vec<I::Item>
    where
        I: Iterator,
        F: FnMut(&I::Item) -> B,
        B: Ord
{
    let first_item = iter.next();

    if !first_item.is_some() {
        return Vec::new();
    }

    /* We are now sure to have an initial value */
    let first_item = first_item.unwrap();
    let mut max_val = f(&first_item);
    let mut max_items = vec![first_item];

    for item in iter {
        let item_val = f(&item);

        if item_val > max_val {
            max_val = item_val;
            max_items.clear();
            max_items.push(item);
        }
        else if item_val == max_val {
            max_items.push(item);
        }
    }

    return max_items;
}

/* Implementation of a stateful map */

pub struct FoldMap<S, I, F> {
    state:     S,
    iter:      I,
    upd_state: F
}

pub fn stateful_map<S, I, F>(init_state: S, iter: I, upd_state: F) -> FoldMap<S, I, F>
    where
        I: Iterator,
        F: FnMut(&S, &I::Item) -> S,
        S: Clone
{
    FoldMap {
        state: init_state,
        iter,
        upd_state
    }
}

impl<S, I, F> Iterator for FoldMap<S, I, F>
    where
        I: Iterator,
        F: FnMut(&S, &I::Item) -> S,
        S: Clone
{
    type Item = (S, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(a) => {
                self.state = (self.upd_state)(&self.state, &a);
                Some((self.state.clone(), a))
            }

            None => None
        }
    }
}
