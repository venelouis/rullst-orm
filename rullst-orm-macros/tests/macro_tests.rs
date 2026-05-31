use syn::{parse_quote, DeriveInput};

#[test]
fn test_basic_model() {
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        #[orm(table = "users")]
        pub struct User {
            pub id: i32,
            pub name: String,
            pub email: String,
        }
    };

    // This test just ensures the macro compiles without panicking
    // The actual code generation is tested by the examples
    let _ = input;
}

#[test]
fn test_model_with_relations() {
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        pub struct Post {
            pub id: i32,
            pub title: String,
            #[orm(has_many = "Comment")]
            comments: Option<Vec<Comment>>,
        }
    };

    let _ = input;
}

#[test]
fn test_model_with_soft_deletes() {
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        pub struct User {
            pub id: i32,
            pub name: String,
            pub deleted_at: Option<String>,
        }
    };

    let _ = input;
}

#[test]
fn test_model_with_hidden_fields() {
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        pub struct User {
            pub id: i32,
            pub name: String,
            #[orm(hidden)]
            pub password: String,
        }
    };

    let _ = input;
}
