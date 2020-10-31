

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
