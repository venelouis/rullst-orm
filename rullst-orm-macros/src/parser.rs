use syn::{Data, DeriveInput, Fields, spanned::Spanned};

/// Validates that a relation attribute has valid syntax
fn validate_relation_attribute(key: &str, value: &str, span: proc_macro2::Span) -> Result<(), syn::Error> {
    match key {
        "has_many" | "has_one" | "belongs_to" | "belongs_to_many" | "morph_many" | "morph_one" => {
            if value.is_empty() {
                return Err(syn::Error::new(span, format!("{} requires a model name", key)));
            }
            // Check if value looks like a valid Rust identifier
            if !value.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                return Err(syn::Error::new(span, format!("{} model name should start with uppercase (PascalCase)", key)));
            }
        }
        "foreign_key" | "related_key" | "pivot_table" | "local_key" | "name"
            if value.is_empty() => {
                return Err(syn::Error::new(span, format!("{} requires a value", key)));
            }
        _ => {}
    }
    Ok(())
}

pub struct ParsedModel {
    pub name: syn::Ident,
    pub table_name: String,
    pub global_scope: String,
    pub before_save: String,
    pub after_save: String,
    pub before_delete: String,
    pub after_delete: String,
    pub after_fetch: String,
    
    pub normal_fields: Vec<syn::Ident>,
    pub hidden_fields: Vec<syn::Ident>,
    pub relations: Vec<ParsedRelation>,
    pub has_soft_deletes: bool,
}

pub struct ParsedRelation {
    pub field_name: syn::Ident,
    pub rel_type: String,
    pub rel_model: String,
    pub foreign_key: String,
    pub local_key: String,
    pub related_key: String,
    pub pivot_table: String,
    pub morph_name: String,
}

pub fn parse(input: &DeriveInput) -> Result<ParsedModel, syn::Error> {
    let name = input.ident.clone();
    let mut table_name = format!("{}s", name.to_string().to_lowercase());
    let mut global_scope = String::new();
    let mut before_save = String::new();
    let mut after_save = String::new();
    let mut before_delete = String::new();
    let mut after_delete = String::new();
    let mut after_fetch = String::new();

    for attr in &input.attrs {
        if attr.path().is_ident("eloquent") {
            let token_str = match attr.meta.require_list() {
                Ok(list) => list.tokens.to_string(),
                Err(_) => continue, // Skip malformed attributes
            };
            for part in token_str.split(',') {
                let parts: Vec<&str> = part.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val = parts[1].trim().trim_matches('"');
                    match key {
                        "table" => table_name = val.to_string(),
                        "global_scope" => global_scope = val.to_string(),
                        "before_save" => before_save = val.to_string(),
                        "after_save" => after_save = val.to_string(),
                        "before_delete" => before_delete = val.to_string(),
                        "after_delete" => after_delete = val.to_string(),
                        "after_fetch" => after_fetch = val.to_string(),
                        _ => {}
                    }
                }
            }
        }
    }

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("Eloquent macro only supports structs with named fields"),
        },
        _ => panic!("Eloquent macro can only be used on structs"),
    };

    let mut normal_fields = vec![];
    let mut hidden_fields = vec![];
    let mut relations = vec![];
    let mut has_soft_deletes = false;

    for field in fields {
        let field_name = match field.ident.as_ref() {
            Some(ident) => ident.clone(),
            None => continue, // Skip fields without identifiers
        };
        let field_name_str = field_name.to_string();
        if field_name_str == "deleted_at" { has_soft_deletes = true; }
        
        let mut is_relation = false;
        let mut rel_type = String::new();
        let mut rel_model = String::new();
        let mut foreign_key = String::new();
        let mut related_key = String::new();
        let mut pivot_table = String::new();
        let mut local_key = "id".to_string();
        let mut morph_name = String::new();
        let mut is_hidden = false;

        for attr in &field.attrs {
            if attr.path().is_ident("eloquent") {
                let token_str = match attr.meta.require_list() {
                    Ok(list) => list.tokens.to_string(),
                    Err(_) => continue, // Skip malformed attributes
                };
                for part in token_str.split(',') {
                    let trimmed = part.trim();
                    if trimmed == "hidden" {
                        is_hidden = true;
                    } else {
                        let parts: Vec<&str> = trimmed.split('=').collect();
                        if parts.len() == 2 {
                            let key = parts[0].trim();
                            let val = parts[1].trim().trim_matches('"');
                            // Validate relation attributes
                            validate_relation_attribute(key, val, field.span())?;
                            match key {
                                "has_many" => { is_relation = true; rel_type = "has_many".to_string(); rel_model = val.to_string(); }
                                "has_one" => { is_relation = true; rel_type = "has_one".to_string(); rel_model = val.to_string(); }
                                "belongs_to" => { is_relation = true; rel_type = "belongs_to".to_string(); rel_model = val.to_string(); }
                                "belongs_to_many" => { is_relation = true; rel_type = "belongs_to_many".to_string(); rel_model = val.to_string(); }
                                "morph_many" => { is_relation = true; rel_type = "morph_many".to_string(); rel_model = val.to_string(); }
                                "morph_one" => { is_relation = true; rel_type = "morph_one".to_string(); rel_model = val.to_string(); }
                                "foreign_key" => foreign_key = val.to_string(),
                                "related_key" => related_key = val.to_string(),
                                "pivot_table" => pivot_table = val.to_string(),
                                "local_key" => local_key = val.to_string(),
                                "name" => morph_name = val.to_string(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if is_relation {
            relations.push(ParsedRelation {
                field_name,
                rel_type,
                rel_model,
                foreign_key,
                local_key,
                related_key,
                pivot_table,
                morph_name,
            });
        } else {
            normal_fields.push(field_name.clone());
            if is_hidden {
                hidden_fields.push(field_name);
            }
        }
    }

    Ok(ParsedModel {
        name,
        table_name,
        global_scope,
        before_save,
        after_save,
        before_delete,
        after_delete,
        after_fetch,
        normal_fields,
        hidden_fields,
        relations,
        has_soft_deletes,
    })
}
