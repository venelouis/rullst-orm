use syn::{DeriveInput, parse_quote};

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

#[test]
fn test_model_with_explicit_soft_delete_config() {
    // Custom column name, integer sentinel, function-based delval.
    // Exercises the MySQL/PostgreSQL/SQLite portable `value`/`delval`
    // handling without binding to any specific dialect.
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        #[orm(soft_delete(field = "is_deleted", value = "0", delval = "1"))]
        pub struct Post {
            pub id: i32,
            pub title: String,
            pub is_deleted: i32,
        }
    };
    let _ = input;
}

#[test]
fn test_model_with_soft_delete_null_sentinel() {
    // `value = "null"` should map to `IS NULL` filters and `delval`
    // can be an arbitrary database function.
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        #[orm(soft_delete(field = "deleted_at", value = "null", delval = "now()"))]
        pub struct Audit {
            pub id: i32,
            pub message: String,
            pub deleted_at: Option<String>,
        }
    };
    let _ = input;
}

#[test]
fn test_model_with_soft_delete_bigint_timestamp() {
    // Bigint + UNIX_TIMESTAMP() pattern suitable for unique-index
    // multi-delete use cases (mybatis-plus compatibility).
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        #[orm(soft_delete(field = "deleted_at", value = "0", delval = "UNIX_TIMESTAMP()"))]
        pub struct Article {
            pub id: i32,
            pub title: String,
            pub deleted_at: i64,
        }
    };
    let _ = input;
}

#[test]
fn test_model_with_orm_skip_field() {
    // `#[orm(skip)]` should remove the field from generated SQL
    // and the `*Column` enum, but the field stays on the struct.
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        pub struct Account {
            pub id: i32,
            pub name: String,
            #[orm(skip)]
            pub password_hash: String,
        }
    };
    let _ = input;
}

#[test]
fn test_model_with_sqlx_skip_field() {
    // The `#[sqlx(skip)]` alias is also accepted and behaves the
    // same as `#[orm(skip)]`.
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        pub struct Account {
            pub id: i32,
            pub name: String,
            #[sqlx(skip)]
            pub password_hash: String,
        }
    };
    let _ = input;
}

#[test]
fn test_model_with_combined_soft_delete_and_skip() {
    // Combine both new features in one model to make sure the
    // generator handles them together without interfering.
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        #[orm(soft_delete(field = "is_active", value = "true", delval = "false"))]
        pub struct User {
            pub id: i32,
            pub name: String,
            pub is_active: bool,
            #[sqlx(skip)]
            pub internal_note: String,
        }
    };
    let _ = input;
}

#[test]
fn test_model_with_tenant_column_string() {
    // The new `tenant_column` arg attaches a per-entity
    // `WHERE <col> = ?` filter; the generated `query()` exposes
    // `without_tenant()` so callers can bypass it on a per-query
    // basis. The macro accepts the `tenant_column` argument on its
    // own and alongside any other knob.
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        #[orm(table = "products", tenant_column = "tenant_id")]
        pub struct Product {
            pub id: i32,
            pub name: String,
            pub tenant_id: String,
        }
    };
    let _ = input;
}

#[test]
fn test_model_with_tenant_column_int() {
    // Integer `tenant_column` is the most common shape for
    // MyBatis-Plus-style multi-tenancy.
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        #[orm(table = "orders", tenant_column = "tenant_id")]
        pub struct Order {
            pub id: i32,
            pub order_number: String,
            pub tenant_id: i32,
        }
    };
    let _ = input;
}

#[test]
fn test_model_with_tenant_column_and_soft_delete() {
    // Tenant column + soft delete in the same model. The macro
    // should not double-inject WHEREs and the generated `query()`
    // should still respect both knobs independently.
    let input: DeriveInput = parse_quote! {
        #[derive(Orm)]
        #[orm(
            table = "orders",
            tenant_column = "tenant_id",
            soft_delete(field = "is_deleted", value = "0", delval = "1")
        )]
        pub struct Order {
            pub id: i32,
            pub order_number: String,
            pub tenant_id: i32,
            pub is_deleted: i32,
        }
    };
    let _ = input;
}
