use crate::parser::ParsedModel;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(parsed: &ParsedModel, relationship_methods: &[TokenStream]) -> TokenStream {
    let name = &parsed.name;
    let table_name = &parsed.table_name;
    let builder_name = quote::format_ident!("{}QueryBuilder", name);
    let observer_trait_name = quote::format_ident!("{}Observer", name);

    let enum_def = generate_column_enum(parsed);
    let json_methods = generate_json_methods(parsed);
    let search_method = generate_search_method(parsed, &builder_name);
    let save_method = generate_save_method(parsed);
    let delete_methods = generate_delete_methods(parsed);
    let query_methods = generate_query_methods(parsed, &builder_name);

    quote! {
        #enum_def

        #[rullst_orm::async_trait]
        impl rullst_orm::RullstModel for #name {
            fn table_name() -> &'static str {
                #table_name
            }
        }

        impl #name {
            #(#relationship_methods)*

            #json_methods

            pub fn observe(observer: std::sync::Arc<dyn #observer_trait_name + Send + Sync>) {
                let list = Self::observers();
                let mut writer = list.write().unwrap_or_else(|poisoned| poisoned.into_inner());
                writer.push(observer);
            }

            fn observers() -> &'static std::sync::RwLock<Vec<std::sync::Arc<dyn #observer_trait_name + Send + Sync>>> {
                static LIST: std::sync::OnceLock<std::sync::RwLock<Vec<std::sync::Arc<dyn #observer_trait_name + Send + Sync>>>> = std::sync::OnceLock::new();
                LIST.get_or_init(|| std::sync::RwLock::new(vec![]))
            }

            #search_method
            #query_methods
            #save_method
            #delete_methods
        }
    }
}

fn generate_column_enum(parsed: &ParsedModel) -> TokenStream {
    let name = &parsed.name;
    let normal_fields = &parsed.normal_fields;
    let column_enum_name = quote::format_ident!("{}Column", name);

    let column_variants: Vec<_> = normal_fields
        .iter()
        .map(|ident| {
            let name_str = ident.to_string();
            let mut chars = name_str.chars();
            let mut camel = match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            };
            camel = camel
                .split('_')
                .map(|s| {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect();
            quote::format_ident!("{}", camel)
        })
        .collect();

    let column_to_string: Vec<_> = normal_fields
        .iter()
        .zip(column_variants.iter())
        .map(|(ident, variant)| {
            let field_name_str = ident.to_string();
            quote! { #column_enum_name::#variant => #field_name_str }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #column_enum_name {
            #(#column_variants),*
        }
        impl #column_enum_name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    #(#column_to_string),*
                }
            }
        }
    }
}

fn generate_json_methods(parsed: &ParsedModel) -> TokenStream {
    let normal_fields = &parsed.normal_fields;
    let hidden_fields = &parsed.hidden_fields;
    let mut relation_field_idents = vec![];
    for rel in &parsed.relations {
        relation_field_idents.push(rel.field_name.clone());
    }

    let mut to_json_fields = vec![];
    for field_name in normal_fields {
        let field_name_str = field_name.to_string();
        if !hidden_fields.contains(field_name) {
            to_json_fields.push(quote! {
                map.insert(#field_name_str.to_string(), rullst_orm::_serde_json::json!(self.#field_name));
            });
        }
    }

    quote! {
        pub fn from_json(json_str: &str) -> Result<Self, rullst_orm::_serde_json::Error> {
            let value: rullst_orm::_serde_json::Value = rullst_orm::_serde_json::from_str(json_str)?;
            Self::from_json_value(value)
        }

        pub fn from_json_value(value: rullst_orm::_serde_json::Value) -> Result<Self, rullst_orm::_serde_json::Error> {
            Ok(Self {
                #(
                    #normal_fields: rullst_orm::_serde_json::from_value(value[stringify!(#normal_fields)].clone())?,
                )*
                #(
                    #relation_field_idents: None,
                )*
            })
        }

        pub fn from_json_array(json_str: &str) -> Result<Vec<Self>, rullst_orm::_serde_json::Error> {
            let value: rullst_orm::_serde_json::Value = rullst_orm::_serde_json::from_str(json_str)?;
            let mut results = vec![];
            if let Some(arr) = value.as_array() {
                for item in arr {
                    results.push(Self::from_json_value(item.clone())?);
                }
            }
            Ok(results)
        }

        pub fn to_cache_json(&self) -> String {
            let mut map = rullst_orm::_serde_json::Map::new();
            #(
                map.insert(stringify!(#normal_fields).to_string(), rullst_orm::_serde_json::json!(self.#normal_fields));
            )*
            rullst_orm::_serde_json::Value::Object(map).to_string()
        }

        pub fn to_cache_json_array(models: &[Self]) -> String {
            let json_values: Vec<rullst_orm::_serde_json::Value> = models.iter().map(|m| {
                let mut map = rullst_orm::_serde_json::Map::new();
                #(
                    map.insert(stringify!(#normal_fields).to_string(), rullst_orm::_serde_json::json!(m.#normal_fields));
                )*
                rullst_orm::_serde_json::Value::Object(map)
            }).collect();
            rullst_orm::_serde_json::Value::Array(json_values).to_string()
        }

        pub fn from_cache_json(json_str: &str) -> Result<Self, rullst_orm::_serde_json::Error> {
            Self::from_json(json_str)
        }

        pub fn from_cache_json_array(json_str: &str) -> Result<Vec<Self>, rullst_orm::_serde_json::Error> {
            Self::from_json_array(json_str)
        }

        pub fn to_json(&self) -> String {
            let mut map = rullst_orm::_serde_json::Map::new();
            #(#to_json_fields)*
            rullst_orm::_serde_json::Value::Object(map).to_string()
        }
    }
}

fn generate_search_method(parsed: &ParsedModel, builder_name: &syn::Ident) -> TokenStream {
    if !parsed.searchable {
        return quote! {};
    }
    let table_name = &parsed.table_name;
    let cols = parsed
        .normal_fields
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>();
    quote! {
        pub async fn search(query: &str) -> #builder_name {
            let mut base_builder = #builder_name::new();
            if let Some(engine) = rullst_orm::scout::get_search_engine() {
                let ids = engine.search(#table_name, query).await.unwrap_or_default();
                if ids.is_empty() {
                    base_builder = base_builder.where_eq("id", 0); // impossible match
                } else {
                    let mut sql_ids = String::new();
                    for (i, id) in ids.iter().enumerate() {
                        sql_ids.push_str(&id.to_string());
                        if i < ids.len() - 1 {
                            sql_ids.push_str(",");
                        }
                    }
                    base_builder = base_builder.where_raw(format!("id IN ({})", sql_ids).as_str(), vec![] as Vec<rullst_orm::RullstValue>);
                }
                return base_builder;
            }

            let driver = rullst_orm::Orm::driver();
            let cast_type = if driver == "mysql" { "CHAR" } else { "TEXT" };
            let like_query = format!("%{}%", query);
            let cols = vec![#(#cols),*];
            let mut raw_parts: Vec<String> = Vec::with_capacity(cols.len());
            for col in &cols {
                raw_parts.push(format!("CAST({} AS {}) LIKE ?", col, cast_type));
            }
            let raw_where = raw_parts.join(" OR ");
            let mut bindings = Vec::with_capacity(cols.len());
            for _ in &cols {
                bindings.push(rullst_orm::RullstValue::String(like_query.clone()));
            }
            base_builder.where_raw(raw_where.as_str(), bindings)
        }
    }
}

fn generate_query_methods(parsed: &ParsedModel, builder_name: &syn::Ident) -> TokenStream {
    let global_scope_logic = if !parsed.global_scope.is_empty() {
        let name = &parsed.name;
        let method = syn::Ident::new(&parsed.global_scope, name.span());
        quote! { builder = builder.#method(); }
    } else {
        quote! {}
    };

    let tenant_scope_logic = if !parsed.tenant_column.is_empty() {
        let col = &parsed.tenant_column;
        quote! {
            if let Some(tenant) = rullst_orm::tenant::get_tenant_id() {
                builder = builder.where_eq(#col, tenant);
            }
        }
    } else {
        quote! {}
    };

    quote! {
        pub fn query() -> #builder_name {
            let mut builder = #builder_name::new();
            #global_scope_logic
            #tenant_scope_logic
            builder
        }

        pub async fn find(id: i32) -> Result<Option<Self>, rullst_orm::Error> {
            Self::query().where_eq("id", id).first().await
        }

        pub async fn find_with_tx(id: i32, tx: &mut rullst_orm::db::Transaction<'static>) -> Result<Option<Self>, rullst_orm::Error> {
            Self::query().where_eq("id", id).first_with_tx(tx).await
        }

        pub async fn all() -> Result<Vec<Self>, rullst_orm::Error> {
            Self::query().get().await
        }

        pub async fn all_with_tx(tx: &mut rullst_orm::db::Transaction<'static>) -> Result<Vec<Self>, rullst_orm::Error> {
            Self::query().get_with_tx(tx).await
        }
    }
}

fn generate_save_method(parsed: &ParsedModel) -> TokenStream {
    let name = &parsed.name;
    let table_name = &parsed.table_name;
    let normal_fields = &parsed.normal_fields;

    let hook_before_save = if !parsed.before_save.is_empty() {
        let method = syn::Ident::new(&parsed.before_save, name.span());
        quote! { self.#method().await?; }
    } else {
        quote! {}
    };
    let hook_after_save = if !parsed.after_save.is_empty() {
        let method = syn::Ident::new(&parsed.after_save, name.span());
        quote! { self.#method().await?; }
    } else {
        quote! {}
    };

    let tenant_set_logic = if !parsed.tenant_column.is_empty() {
        let col_ident = syn::Ident::new(&parsed.tenant_column, name.span());
        quote! {
            if let Some(tenant) = rullst_orm::tenant::get_tenant_id() {
                if let Ok(val) = tenant.try_into() {
                    self.#col_ident = val;
                }
            }
        }
    } else {
        quote! {}
    };

    let audit_before_update = if parsed.auditable {
        quote! {
            let old_model_for_audit = if !is_new {
                let pool = rullst_orm::Orm::read_pool();
                let driver = rullst_orm::Orm::driver();
                let query = if driver == "postgres" {
                    format!("SELECT * FROM {} WHERE id = $1", #table_name)
                } else {
                    format!("SELECT * FROM {} WHERE id = ?", #table_name)
                };
                rullst_orm::_sqlx::query_as::<_, Self>(rullst_orm::_sqlx::AssertSqlSafe(query.as_str()))
                    .bind(self.id)
                    .fetch_optional(pool)
                    .await?
            } else {
                None
            };
        }
    } else {
        quote! {}
    };

    let audit_after_save = if parsed.auditable {
        quote! {
            if is_new {
                let _ = rullst_orm::audit::log_audit(
                    #table_name,
                    self.id,
                    "created",
                    None,
                    Some(self.to_json())
                ).await;
            } else if let Some(old_model) = old_model_for_audit {
                let _ = rullst_orm::audit::log_audit_diff(
                    #table_name,
                    self.id,
                    "updated",
                    &old_model.to_json(),
                    &self.to_json()
                ).await;
            }
        }
    } else {
        quote! {}
    };

    let scout_update = if parsed.searchable {
        quote! {
            if let Some(engine) = rullst_orm::scout::get_search_engine() {
                let payload: rullst_orm::_serde_json::Value = match rullst_orm::_serde_json::from_str(&self.to_json()) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[rullst-orm] Scout: failed to serialize model {} (id={}) to JSON: {e}", #table_name, self.id);
                        rullst_orm::_serde_json::Value::Null
                    }
                };
                let _ = engine.update(#table_name, self.id, payload).await;
            }
        }
    } else {
        quote! {}
    };

    let mut insert_columns = vec![];
    let mut insert_placeholders = vec![];
    let mut bind_inserts = vec![];

    let mut update_sets = vec![];
    let mut bind_updates = vec![];

    for field_name in normal_fields {
        let field_name_str = field_name.to_string();
        if field_name_str != "id" {
            insert_columns.push(field_name_str.clone());
            insert_placeholders.push("?");
            bind_inserts.push(quote! { .bind(self.#field_name.clone()) });

            update_sets.push(format!("{} = ?", field_name_str));
            bind_updates.push(quote! { .bind(self.#field_name.clone()) });
        }
    }

    let insert_columns_str = insert_columns.join(", ");
    let insert_placeholders_str = insert_placeholders.join(", ");
    let update_sets_str = update_sets.join(", ");

    quote! {
        pub async fn save(&mut self) -> Result<(), rullst_orm::Error> {
            let pool = rullst_orm::Orm::pool();
            self.save_with_tx_internal(pool).await
        }

        pub async fn save_with_tx(&mut self, tx: &mut rullst_orm::db::Transaction<'static>) -> Result<(), rullst_orm::Error> {
            self.save_with_tx_internal(&mut **tx).await
        }

        async fn save_with_tx_internal<'e, E>(&mut self, executor: E) -> Result<(), rullst_orm::Error>
        where E: rullst_orm::_sqlx::Executor<'e, Database = rullst_orm::RullstDatabase>
        {
            let is_new = self.id == 0;
            if is_new {
                #tenant_set_logic
            }
            #audit_before_update
            #hook_before_save
            let observers = {
                let list = Self::observers().read().unwrap_or_else(|poisoned| poisoned.into_inner());
                list.clone()
            };
            for obs in &observers {
                obs.saving(self).await?;
            }
            if self.id == 0 {
                for obs in &observers {
                    obs.creating(self).await?;
                }
                let driver = rullst_orm::Orm::driver();
                if driver == "postgres" || driver == "sqlite" {
                    use rullst_orm::_sqlx::Execute;
                    let mut final_sql = format!("INSERT INTO {} ({}) VALUES ({}) RETURNING id", #table_name, #insert_columns_str, #insert_placeholders_str);
                    if driver == "postgres" {
                        let mut replaced = String::with_capacity(final_sql.len());
                        let mut idx = 1;
                        for c in final_sql.chars() {
                            if c == '?' {
                                replaced.push_str(&format!("${}", idx));
                                idx += 1;
                            } else {
                                replaced.push(c);
                            }
                        }
                        final_sql = replaced;
                    }
                    if rullst_orm::schema::is_query_log_enabled() {
                        println!("[SQL Debug] {:?}", final_sql);
                    }
                    let query = rullst_orm::_sqlx::query(rullst_orm::_sqlx::AssertSqlSafe(final_sql.as_str()));
                    let row = query
                        #(#bind_inserts)*
                        .fetch_one(executor)
                        .await?;
                    self.id = rullst_orm::_sqlx::Row::try_get(&row, "id")?;
                } else {
                    use rullst_orm::_sqlx::Execute;
                    let mut final_sql = format!("INSERT INTO {} ({}) VALUES ({})", #table_name, #insert_columns_str, #insert_placeholders_str);
                    if rullst_orm::schema::is_query_log_enabled() {
                        println!("[SQL Debug] {:?}", final_sql);
                    }
                    let query = rullst_orm::_sqlx::query(rullst_orm::_sqlx::AssertSqlSafe(final_sql.as_str()));
                    let result = query
                        #(#bind_inserts)*
                        .execute(executor)
                        .await?;
                    self.id = {
                        use rullst_orm::database::QueryResultExt;
                        result.get_last_insert_id() as i32
                    }
                }
                for obs in &observers {
                    obs.created(self).await?;
                }
            } else {
                for obs in &observers {
                    obs.updating(self).await?;
                }
                use rullst_orm::_sqlx::Execute;
                let mut final_sql = format!("UPDATE {} SET {} WHERE id = ?", #table_name, #update_sets_str);
                if rullst_orm::Orm::driver() == "postgres" {
                    let mut replaced = String::with_capacity(final_sql.len());
                    let mut idx = 1;
                    for c in final_sql.chars() {
                        if c == '?' {
                            replaced.push_str(&format!("${}", idx));
                            idx += 1;
                        } else {
                            replaced.push(c);
                        }
                    }
                    final_sql = replaced;
                }
                if rullst_orm::schema::is_query_log_enabled() {
                    println!("[SQL Debug] {:?} | ID: {}", final_sql, self.id);
                }
                let query = rullst_orm::_sqlx::query(rullst_orm::_sqlx::AssertSqlSafe(final_sql.as_str()));
                query
                    #(#bind_updates)*
                    .bind(self.id)
                    .execute(executor)
                    .await?;
                for obs in &observers {
                    obs.updated(self).await?;
                }
            }
            for obs in &observers {
                obs.saved(self).await?;
            }
            #[cfg(feature = "redis")]
            {
                use rullst_orm::_redis::AsyncCommands;
                let mut conn = rullst_orm::Orm::redis_manager()?;
                let payload = self.to_json();
                if is_new {
                    let topic = format!("orm:events:{}:created", #table_name);
                    let _: Result<usize, _> = conn.publish(&topic, &payload).await;
                } else {
                    let topic = format!("orm:events:{}:updated", #table_name);
                    let _: Result<usize, _> = conn.publish(&topic, &payload).await;
                }
                let topic = format!("orm:events:{}:saved", #table_name);
                let _: Result<usize, _> = conn.publish(&topic, &payload).await;
            }
            #audit_after_save
            #scout_update
            #hook_after_save
            Ok(())
        }
    }
}

fn generate_delete_methods(parsed: &ParsedModel) -> TokenStream {
    let name = &parsed.name;
    let table_name = &parsed.table_name;
    let has_soft_deletes = parsed.has_soft_deletes;

    let hook_before_delete = if !parsed.before_delete.is_empty() {
        let method = syn::Ident::new(&parsed.before_delete, name.span());
        quote! { self.#method().await?; }
    } else {
        quote! {}
    };
    let hook_after_delete = if !parsed.after_delete.is_empty() {
        let method = syn::Ident::new(&parsed.after_delete, name.span());
        quote! { self.#method().await?; }
    } else {
        quote! {}
    };

    let audit_after_delete = if parsed.auditable {
        quote! {
            let _ = rullst_orm::audit::log_audit(
                #table_name,
                self.id,
                "deleted",
                Some(self.to_json()),
                None
            ).await;
        }
    } else {
        quote! {}
    };

    let scout_delete = if parsed.searchable {
        quote! {
            if let Some(engine) = rullst_orm::scout::get_search_engine() {
                let _ = engine.delete(#table_name, self.id).await;
            }
        }
    } else {
        quote! {}
    };

    let delete_logic = if has_soft_deletes {
        quote! {
            let driver = rullst_orm::Orm::driver();
            let query = if driver == "postgres" {
                format!("UPDATE {} SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1", #table_name)
            } else {
                format!("UPDATE {} SET deleted_at = CURRENT_TIMESTAMP WHERE id = ?", #table_name)
            };
        }
    } else {
        quote! {
            let driver = rullst_orm::Orm::driver();
            let query = if driver == "postgres" {
                format!("DELETE FROM {} WHERE id = $1", #table_name)
            } else {
                format!("DELETE FROM {} WHERE id = ?", #table_name)
            };
        }
    };

    quote! {
        pub async fn delete(&self) -> Result<(), rullst_orm::Error> {
            let pool = rullst_orm::Orm::pool();
            self.delete_with_tx_internal(pool).await
        }

        pub async fn delete_with_tx(&self, tx: &mut rullst_orm::db::Transaction<'static>) -> Result<(), rullst_orm::Error> {
            self.delete_with_tx_internal(&mut **tx).await
        }

        async fn delete_with_tx_internal<'e, E>(&self, executor: E) -> Result<(), rullst_orm::Error>
        where E: rullst_orm::_sqlx::Executor<'e, Database = rullst_orm::RullstDatabase>
        {
            #hook_before_delete
            let observers = {
                let list = Self::observers().read().unwrap_or_else(|poisoned| poisoned.into_inner());
                list.clone()
            };
            for obs in &observers {
                obs.deleting(self).await?;
            }
            #delete_logic
            if rullst_orm::schema::is_query_log_enabled() {
                println!("[SQL Debug] {:?} | ID: {}", query, self.id);
            }
            rullst_orm::_sqlx::query(rullst_orm::_sqlx::AssertSqlSafe(query.as_str())).bind(self.id).execute(executor).await?;
            for obs in &observers {
                obs.deleted(self).await?;
            }
            #[cfg(feature = "redis")]
            {
                use rullst_orm::_redis::AsyncCommands;
                let mut conn = rullst_orm::Orm::redis_manager()?;
                let payload = self.to_json();
                let topic = format!("orm:events:{}:deleted", #table_name);
                let _: Result<usize, _> = conn.publish(&topic, &payload).await;
            }
            #audit_after_delete
            #scout_delete
            #hook_after_delete
            Ok(())
        }

        pub async fn restore(&self) -> Result<(), rullst_orm::Error> {
            if #has_soft_deletes {
                let pool = rullst_orm::Orm::pool();
                use rullst_orm::_sqlx::query_builder::QueryBuilder;
                let mut query_builder = QueryBuilder::new("UPDATE ");
                query_builder.push(#table_name);
                if rullst_orm::Orm::driver() == "postgres" {
                    query_builder.push(" SET deleted_at = NULL WHERE id = $1");
                } else {
                    query_builder.push(" SET deleted_at = NULL WHERE id = ?");
                }
                let query = query_builder.build();
                query.bind(self.id).execute(pool).await?;
            }
            Ok(())
        }

        pub async fn force_delete(&self) -> Result<(), rullst_orm::Error> {
            let pool = rullst_orm::Orm::pool();
            use rullst_orm::_sqlx::query_builder::QueryBuilder;
            let mut query_builder = QueryBuilder::new("DELETE FROM ");
            query_builder.push(#table_name);
            if rullst_orm::Orm::driver() == "postgres" {
                query_builder.push(" WHERE id = $1");
            } else {
                query_builder.push(" WHERE id = ?");
            }
            let query = query_builder.build();
            query.bind(self.id).execute(pool).await?;
            Ok(())
        }
    }
}
