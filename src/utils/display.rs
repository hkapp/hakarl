use crate::utils::{Stack, KeyValue};

/* Vec::join for Iter */

#[allow(dead_code)]
pub fn join<I>(iter: I, sep: &str) -> String
    where
        I: Iterator<Item = String>
{
    enclosed_join(iter, "", sep, "")
}

pub fn enclosed_join<I>(mut iter: I, opening: &str, sep: &str, closing: &str) -> String
    where
        I: Iterator<Item = String>
{
    let mut res = String::from(opening);
    if let Some(first_item) = iter.next() {
        res.push_str(&first_item);
    }
    for item in iter {
        res.push_str(sep);
        res.push_str(&item);
    }
    res.push_str(closing);

    return res;
}

/* JsonBuilder */

type Property<T> = KeyValue<String, T>;

type FlatProp = Property<String>;
type NestedProp = Property<FlatJson>;

type FlatJson = Vec<FlatProp>;

pub struct JsonBuilder {
    root:       FlatJson,
    open_jsons: Stack<NestedProp>,
}

impl JsonBuilder {
    pub fn new() -> Self {
        JsonBuilder {
            root:       Vec::new(),
            open_jsons: Stack::new(),
        }
    }

    fn curr_json(&mut self) -> &mut FlatJson {
        let open_count = self.open_jsons.len();
        match open_count {
            0 => &mut self.root,
            _ => &mut self.open_jsons[open_count - 1].value,
        }
    }

    pub fn push(&mut self, name: String, value: String) {
        let new_prop = FlatProp { key: name, value };
        self.curr_json().push(new_prop);
    }

    pub fn open_rec(&mut self, name: String) {
        let new_open_json = NestedProp { key: name, value: Vec::new() };
        self.open_jsons.push(new_open_json);
    }

    pub fn close_rec(&mut self) {
        let json_to_close = self.open_jsons.pop()
                                            .expect("JsonBuilder: no open json to close");
        let NestedProp { key: prop_name, value: json } = json_to_close;
        let json_str = Self::format(json);

        self.push(prop_name, json_str);
    }

    fn format(json: FlatJson) -> String {
        fn format_property(prop: FlatProp) -> String {
            format!("{} : {}", prop.key, prop.value)
        }

        enclosed_join(json.into_iter().map(format_property),
                      "{",
                      ", ",
                      "}")
    }

    pub fn to_string(mut self) -> Result<String, String> {
        if !self.open_jsons.is_empty() {
            Err(
                format!("Nested json field \"{}\" is still open",
                        self.open_jsons.pop().unwrap().key))
        }
        else {
            Ok(Self::format(self.root))
        }
    }
}
