use sqlx::{any::AnyArguments, Arguments};

pub fn test() {
    let mut args = AnyArguments::default();
    let _ = args.add(10_i32);
    let _ = args.add("test".to_string());
}
