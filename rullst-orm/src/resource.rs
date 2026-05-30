use serde_json::Value;

pub trait ApiResource {
    fn to_array(&self) -> Value;
}

pub struct JsonResource<'a, T: ApiResource> {
    pub data: &'a T,
}

impl<'a, T: ApiResource> JsonResource<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self { data }
    }

    pub fn resolve(&self) -> Value {
        self.data.to_array()
    }
}

pub struct ResourceCollection<'a, T: ApiResource> {
    pub data: &'a [T],
}

impl<'a, T: ApiResource> ResourceCollection<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }

    pub fn resolve(&self) -> Value {
        let array: Vec<Value> = self.data.iter().map(|item| item.to_array()).collect();
        serde_json::json!(array)
    }
}
