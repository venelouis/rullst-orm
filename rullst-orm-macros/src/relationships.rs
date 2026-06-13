use crate::parser::ParsedModel;
use proc_macro2::TokenStream;
use quote::quote;

pub struct GeneratedRelationships {
    pub flags: Vec<TokenStream>,
    pub inits: Vec<TokenStream>,
    pub methods: Vec<TokenStream>,
    pub model_methods: Vec<TokenStream>,
    pub eager_loads: TokenStream,
}

pub fn generate(parsed: &ParsedModel) -> GeneratedRelationships {
    let mut flags = vec![];
    let mut inits = vec![];
    let mut methods = vec![];
    let mut model_methods = vec![];

    let name = &parsed.name;

    for rel in &parsed.relations {
        let field_name = &rel.field_name;
        let rel_type = &rel.rel_type;
        let rel_model = &rel.rel_model;
        let foreign_key = &rel.foreign_key;
        let local_key = &rel.local_key;
        let related_key = &rel.related_key;
        let pivot_table = &rel.pivot_table;
        let morph_name = &rel.morph_name;

        let load_flag_ident = quote::format_ident!("load_{}", field_name);
        let filter_flag_ident = quote::format_ident!("filter_{}", field_name);
        let rel_model_builder_ident = quote::format_ident!("{}QueryBuilder", rel_model);

        flags.push(quote! {
            pub #load_flag_ident: bool,
            pub #filter_flag_ident: Option<std::sync::Arc<dyn Fn(#rel_model_builder_ident) -> #rel_model_builder_ident + Send + Sync>>,
        });
        inits.push(quote! {
            #load_flag_ident: false,
            #filter_flag_ident: None,
        });

        let with_method_ident = quote::format_ident!("with_{}", field_name);
        let with_constrained_method_ident = quote::format_ident!("with_{}_constrained", field_name);
        methods.push(quote! {
            pub fn #with_method_ident(mut self) -> Self {
                self.#load_flag_ident = true;
                self
            }
            pub fn #with_constrained_method_ident<F>(mut self, filter: F) -> Self
            where F: Fn(#rel_model_builder_ident) -> #rel_model_builder_ident + Send + Sync + 'static {
                self.#load_flag_ident = true;
                self.#filter_flag_ident = Some(std::sync::Arc::new(filter));
                self
            }
        });

        let rel_model_ident = syn::Ident::new(rel_model, field_name.span());
        let method_name = quote::format_ident!("{}", field_name);
        let method_name_constrained = quote::format_ident!("{}_constrained", field_name);
        let fk_ident = quote::format_ident!(
            "{}",
            if foreign_key.is_empty() {
                format!("{}_id", name.to_string().to_lowercase())
            } else {
                foreign_key.clone()
            }
        );
        let lk_ident = quote::format_ident!(
            "{}",
            if local_key.is_empty() {
                "id".to_string()
            } else {
                local_key.clone()
            }
        );
        let pk_ident = quote::format_ident!(
            "{}",
            if related_key.is_empty() {
                "id".to_string()
            } else {
                related_key.clone()
            }
        );

        if rel_type == "has_many" {
            model_methods.push(quote! {
                pub fn #method_name(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        #rel_model_ident::query().where_eq(stringify!(#fk_ident), self.#lk_ident.clone()).get().await
                    })
                }
                pub fn #method_name_constrained(&self, modifier: std::sync::Arc<dyn Fn(#rel_model_builder_ident) -> #rel_model_builder_ident + Send + Sync>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        let mut q = #rel_model_ident::query().where_eq(stringify!(#fk_ident), self.#lk_ident.clone());
                        q = modifier(q);
                        q.get().await
                    })
                }
            });
        } else if rel_type == "has_one" {
            model_methods.push(quote! {
                pub fn #method_name(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        #rel_model_ident::query().where_eq(stringify!(#fk_ident), self.#lk_ident.clone()).first().await
                    })
                }
                pub fn #method_name_constrained(&self, modifier: std::sync::Arc<dyn Fn(#rel_model_builder_ident) -> #rel_model_builder_ident + Send + Sync>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        let mut q = #rel_model_ident::query().where_eq(stringify!(#fk_ident), self.#lk_ident.clone());
                        q = modifier(q);
                        q.first().await
                    })
                }
            });
        } else if rel_type == "belongs_to" {
            model_methods.push(quote! {
                pub fn #method_name(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        #rel_model_ident::query().where_eq(stringify!(#pk_ident), self.#fk_ident.clone()).first().await
                    })
                }
                pub fn #method_name_constrained(&self, modifier: std::sync::Arc<dyn Fn(#rel_model_builder_ident) -> #rel_model_builder_ident + Send + Sync>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        let mut q = #rel_model_ident::query().where_eq(stringify!(#pk_ident), self.#fk_ident.clone());
                        q = modifier(q);
                        q.first().await
                    })
                }
            });
        } else if rel_type == "morph_many" {
            let morph_type_ident = quote::format_ident!("{}_type", morph_name);
            let morph_id_ident = quote::format_ident!("{}_id", morph_name);
            model_methods.push(quote! {
                pub fn #method_name(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        #rel_model_ident::query()
                            .where_eq(stringify!(#morph_id_ident), self.#lk_ident.clone())
                            .where_eq(stringify!(#morph_type_ident), stringify!(#name))
                            .get().await
                    })
                }
                pub fn #method_name_constrained(&self, modifier: std::sync::Arc<dyn Fn(#rel_model_builder_ident) -> #rel_model_builder_ident + Send + Sync>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        let mut q = #rel_model_ident::query()
                            .where_eq(stringify!(#morph_id_ident), self.#lk_ident.clone())
                            .where_eq(stringify!(#morph_type_ident), stringify!(#name));
                        q = modifier(q);
                        q.get().await
                    })
                }
            });
        } else if rel_type == "morph_one" {
            let morph_type_ident = quote::format_ident!("{}_type", morph_name);
            let morph_id_ident = quote::format_ident!("{}_id", morph_name);
            model_methods.push(quote! {
                pub fn #method_name(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        #rel_model_ident::query()
                            .where_eq(stringify!(#morph_id_ident), self.#lk_ident.clone())
                            .where_eq(stringify!(#morph_type_ident), stringify!(#name))
                            .first().await
                    })
                }
                pub fn #method_name_constrained(&self, modifier: std::sync::Arc<dyn Fn(#rel_model_builder_ident) -> #rel_model_builder_ident + Send + Sync>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        let mut q = #rel_model_ident::query()
                            .where_eq(stringify!(#morph_id_ident), self.#lk_ident.clone())
                            .where_eq(stringify!(#morph_type_ident), stringify!(#name));
                        q = modifier(q);
                        q.first().await
                    })
                }
            });
        } else if rel_type == "belongs_to_many" {
            let pivot_fk = format!("{}.{}", pivot_table, foreign_key);
            let pivot_rk = format!("{}.{}", pivot_table, related_key);
            model_methods.push(quote! {
                pub fn #method_name(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        let related_pk = format!("{}.{}", #rel_model_ident::table_name(), "id");
                        let select_raw = format!("{}.*", #rel_model_ident::table_name());
                        #rel_model_ident::query()
                            .select_raw(&select_raw)
                            .join(#pivot_table, &related_pk, "=", #pivot_rk)
                            .where_eq(&#pivot_fk, self.#lk_ident.clone())
                            .get().await
                    })
                }
                pub fn #method_name_constrained(&self, modifier: std::sync::Arc<dyn Fn(#rel_model_builder_ident) -> #rel_model_builder_ident + Send + Sync>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<#rel_model_ident>, rullst_orm::Error>> + Send + '_>> {
                    Box::pin(async move {
                        let related_pk = format!("{}.{}", #rel_model_ident::table_name(), "id");
                        let select_raw = format!("{}.*", #rel_model_ident::table_name());
                        let mut q = #rel_model_ident::query()
                            .select_raw(&select_raw)
                            .join(#pivot_table, &related_pk, "=", #pivot_rk)
                            .where_eq(&#pivot_fk, self.#lk_ident.clone());
                        q = modifier(q);
                        q.get().await
                    })
                }
            });
        }
    }

    let eager_loads_logic: Vec<_> = parsed.relations.iter().map(|rel| {
        let field_name = &rel.field_name;
        let rel_type = &rel.rel_type;
        let rel_model = &rel.rel_model;
        let foreign_key = &rel.foreign_key;
        let local_key = &rel.local_key;
        let related_key = &rel.related_key;
        let morph_name = &rel.morph_name;
        let pivot_table = &rel.pivot_table;

        let load_flag = quote::format_ident!("load_{}", field_name);
        let filter_flag = quote::format_ident!("filter_{}", field_name);
        let method_name = quote::format_ident!("{}", field_name);

        let rel_model_ident = syn::Ident::new(rel_model, field_name.span());
        let fk_ident = quote::format_ident!("{}", if foreign_key.is_empty() { format!("{}_id", name.to_string().to_lowercase()) } else { foreign_key.clone() });
        let lk_ident = quote::format_ident!("{}", if local_key.is_empty() { "id".to_string() } else { local_key.clone() });
        let pk_ident = quote::format_ident!("{}", if related_key.is_empty() { "id".to_string() } else { related_key.clone() });

        if rel_type == "has_many" {
            quote! {
                if self.#load_flag {
                    let parent_ids: Vec<_> = results.iter().map(|m| m.#lk_ident.clone()).collect();
                    if !parent_ids.is_empty() {
                        let mut query = #rel_model_ident::query().where_in(stringify!(#fk_ident), parent_ids);
                        if let Some(ref filter) = self.#filter_flag {
                            query = filter(query);
                        }
                        let all_related = Box::pin(query.get()).await?;
                        let mut map = std::collections::HashMap::with_capacity(all_related.len());
                        for rel in all_related {
                            map.entry(rel.#fk_ident.clone()).or_insert_with(Vec::new).push(rel);
                        }

                        for model in &mut results {
                            let matching = map.remove(&model.#lk_ident).unwrap_or_default();
                            model.#method_name = Some(matching);
                        }
                    }
                }
            }
        } else if rel_type == "has_one" {
            quote! {
                if self.#load_flag {
                    let parent_ids: Vec<_> = results.iter().map(|m| m.#lk_ident.clone()).collect();
                    if !parent_ids.is_empty() {
                        let mut query = #rel_model_ident::query().where_in(stringify!(#fk_ident), parent_ids);
                        if let Some(ref filter) = self.#filter_flag {
                            query = filter(query);
                        }
                        let all_related = Box::pin(query.get()).await?;
                        let mut map = std::collections::HashMap::with_capacity(all_related.len());
                        for rel in all_related {
                            map.entry(rel.#fk_ident.clone()).or_insert(rel);
                        }

                        for model in &mut results {
                            model.#method_name = map.remove(&model.#lk_ident);
                        }
                    }
                }
            }
        } else if rel_type == "belongs_to" {
            quote! {
                if self.#load_flag {
                    let parent_ids: Vec<_> = results.iter().map(|m| m.#fk_ident.clone()).collect();
                    if !parent_ids.is_empty() {
                        let mut query = #rel_model_ident::query().where_in(stringify!(#pk_ident), parent_ids);
                        if let Some(ref filter) = self.#filter_flag {
                            query = filter(query);
                        }
                        let all_related = Box::pin(query.get()).await?;
                        let mut map = std::collections::HashMap::with_capacity(all_related.len());
                        for rel in all_related {
                            map.entry(rel.#pk_ident.clone()).or_insert(rel);
                        }

                        for model in &mut results {
                            model.#method_name = map.remove(&model.#fk_ident);
                        }
                    }
                }
            }
        } else {
            let morph_type_ident = quote::format_ident!("{}_type", morph_name);
            let morph_id_ident = quote::format_ident!("{}_id", morph_name);

            if rel_type == "morph_many" {
                // Batch load: one query with WHERE morph_id IN (...) AND morph_type = 'Name'
                // eliminates the previous N+1 pattern (one query per parent model).
                quote! {
                    if self.#load_flag {
                        let parent_ids: Vec<_> = results.iter().map(|m| m.#lk_ident.clone()).collect();
                        if !parent_ids.is_empty() {
                            let mut query = #rel_model_ident::query()
                                .where_in(stringify!(#morph_id_ident), parent_ids)
                                .where_eq(stringify!(#morph_type_ident), stringify!(#name));
                            if let Some(ref filter) = self.#filter_flag {
                                query = filter(query);
                            }
                            let all_related = Box::pin(query.get()).await?;
                            let mut map = std::collections::HashMap::with_capacity(all_related.len());
                            for rel in all_related {
                                map.entry(rel.#morph_id_ident.clone()).or_insert_with(Vec::new).push(rel);
                            }
                            for model in &mut results {
                                let matching = map.remove(&model.#lk_ident).unwrap_or_default();
                                model.#method_name = Some(matching);
                            }
                        }
                    }
                }
            } else if rel_type == "morph_one" {
                // Batch load: one query for all parents, then distribute.
                quote! {
                    if self.#load_flag {
                        let parent_ids: Vec<_> = results.iter().map(|m| m.#lk_ident.clone()).collect();
                        if !parent_ids.is_empty() {
                            let mut query = #rel_model_ident::query()
                                .where_in(stringify!(#morph_id_ident), parent_ids)
                                .where_eq(stringify!(#morph_type_ident), stringify!(#name));
                            if let Some(ref filter) = self.#filter_flag {
                                query = filter(query);
                            }
                            let all_related = Box::pin(query.get()).await?;
                            let mut map = std::collections::HashMap::with_capacity(all_related.len());
                            for rel in all_related {
                                map.entry(rel.#morph_id_ident.clone()).or_insert(rel);
                            }
                            for model in &mut results {
                                model.#method_name = map.remove(&model.#lk_ident);
                            }
                        }
                    }
                }
            } else {
                // Batch load belongs_to_many: 2 queries for any collection size.
                // Q1: SELECT parent_fk, related_fk FROM pivot WHERE parent_fk IN (...)
                // Q2: SELECT * FROM related_table WHERE id IN (unique_related_ids)
                // Then distribute in memory. No N+1.
                quote! {
                    if self.#load_flag {
                        let parent_ids: Vec<i32> = results.iter().map(|m| m.#lk_ident).collect();
                        if !parent_ids.is_empty() {
                            let pool = rullst_orm::Orm::read_pool();
                            let driver = rullst_orm::Orm::driver();
                            // Q1: pivot table pairs
                            let placeholders_str = vec!["?"; parent_ids.len()].join(", ");
                            let mut pivot_sql = format!(
                                "SELECT {fk}, {rk} FROM {pt} WHERE {fk} IN ({ph})",
                                fk = #foreign_key,
                                rk = #related_key,
                                pt = #pivot_table,
                                ph = placeholders_str,
                            );
                            if driver == "postgres" {
                                use std::fmt::Write;
                                let parts: Vec<&str> = pivot_sql.split('?').collect();
                                let mut pg = String::with_capacity(pivot_sql.len() + parts.len() * 2);
                                for (i, part) in parts.iter().enumerate() {
                                    pg.push_str(part);
                                    if i < parts.len() - 1 {
                                        write!(pg, "${}", i + 1).unwrap();
                                    }
                                }
                                pivot_sql = pg;
                            }
                            let mut pivot_query = rullst_orm::_sqlx::query_as::<_, (i32, i32)>(
                                rullst_orm::_sqlx::AssertSqlSafe(pivot_sql.as_str())
                            );
                            for id in &parent_ids {
                                pivot_query = pivot_query.bind(*id);
                            }
                            let pivot_pairs: Vec<(i32, i32)> = pivot_query.fetch_all(pool).await?;

                            if !pivot_pairs.is_empty() {
                                // Deduplicate related IDs for Q2
                                let mut related_ids: Vec<i32> = pivot_pairs.iter().map(|(_, rid)| *rid).collect();
                                related_ids.sort_unstable();
                                related_ids.dedup();
                                let related_ids_len = related_ids.len();

                                let mut query = #rel_model_ident::query().where_in("id", related_ids);
                                if let Some(ref filter) = self.#filter_flag {
                                    query = filter(query);
                                }
                                let all_related: Vec<#rel_model_ident> = Box::pin(query.get()).await?;

                                // related_id -> model lookup
                                let mut related_map: std::collections::HashMap<i32, #rel_model_ident> =
                                    all_related.into_iter().map(|m| (m.id, m)).collect();

                                // Build parent_id -> Vec<model> from pivot pairs
                                let mut parent_to_related: std::collections::HashMap<i32, Vec<#rel_model_ident>> =
                                    std::collections::HashMap::with_capacity(results.len());
                                let mut related_counts = std::collections::HashMap::with_capacity(related_ids_len);
                                for (_, related_id) in &pivot_pairs {
                                    *related_counts.entry(*related_id).or_insert(0) += 1;
                                }

                                for (parent_id, related_id) in &pivot_pairs {
                                    if let Some(count) = related_counts.get_mut(related_id) {
                                        if *count == 1 {
                                            if let Some(m) = related_map.remove(related_id) {
                                                parent_to_related
                                                    .entry(*parent_id)
                                                    .or_insert_with(Vec::new)
                                                    .push(m);
                                            }
                                        } else {
                                            if let Some(m) = related_map.get(related_id) {
                                                parent_to_related
                                                    .entry(*parent_id)
                                                    .or_insert_with(Vec::new)
                                                    .push(m.clone());
                                            }
                                            *count -= 1;
                                        }
                                    }
                                }

                                for model in &mut results {
                                    model.#method_name = parent_to_related.remove(&model.#lk_ident);
                                }
                            }
                        }
                    }
                }
            }
        }

    }).collect();

    GeneratedRelationships {
        flags,
        inits,
        methods,
        model_methods,
        eager_loads: quote! { #(#eager_loads_logic)* },
    }
}
