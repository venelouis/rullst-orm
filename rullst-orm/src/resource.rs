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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct DummyResource {
        value: i32,
    }

    impl ApiResource for DummyResource {
        fn to_array(&self) -> Value {
            json!({ "value": self.value })
        }
    }

    #[test]
    fn test_json_resource_new_stores_data() {
        let r = DummyResource { value: 42 };
        let jr = JsonResource::new(&r);
        assert_eq!(jr.data.value, 42);
    }

    #[test]
    fn test_json_resource_resolve() {
        let r = DummyResource { value: 7 };
        let jr = JsonResource::new(&r);
        let v = jr.resolve();
        assert_eq!(v["value"], 7);
    }

    #[test]
    fn test_resource_collection_new_stores_slice() {
        let items = vec![DummyResource { value: 1 }, DummyResource { value: 2 }];
        let rc = ResourceCollection::new(&items);
        assert_eq!(rc.data.len(), 2);
    }

    #[test]
    fn test_resource_collection_resolve_returns_array() {
        let items = vec![DummyResource { value: 10 }, DummyResource { value: 20 }];
        let rc = ResourceCollection::new(&items);
        let v = rc.resolve();
        assert!(v.is_array());
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["value"], 10);
        assert_eq!(arr[1]["value"], 20);
    }

    #[test]
    fn test_resource_collection_empty() {
        let items: Vec<DummyResource> = vec![];
        let rc = ResourceCollection::new(&items);
        let v = rc.resolve();
        assert_eq!(v.as_array().unwrap().len(), 0);
    }
}
