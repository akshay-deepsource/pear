use serde_json::{Map, Value};

pub fn get_str(value: &'static Value, id: &str) -> Option<&'static str> {
    value.get(id)?.as_str()
}

pub fn get_bool(value: &'static Value, id: &str) -> Option<bool> {
    value.get(id)?.as_bool()
}

pub fn get_vec(value: &'static Value, id: &str) -> Option<&'static Vec<Value>> {
    value.get(id)?.as_array()
}

pub fn get_map(value: &'static Value, id: &str) -> Option<&'static Map<String, Value>> {
    value.get(id)?.as_object()
}
