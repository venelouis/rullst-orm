extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod parser;
mod builder;
mod relationships;
mod models;
mod factory_observer;

#[proc_macro_derive(Eloquent, attributes(eloquent))]
pub fn eloquent_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Parse the input
    let parsed = match parser::parse(&input) {
        Ok(p) => p,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };
    
    // Generate relationships
    let rels = relationships::generate(&parsed);
    
    // Generate the builder
    let builder_code = builder::generate(
        &parsed, 
        &rels.flags, 
        &rels.inits, 
        &rels.methods, 
        &rels.eager_loads
    );
    
    // Generate factory and observers
    let factory_observer_code = factory_observer::generate(&parsed);
    
    // Generate the model impl
    let model_code = models::generate(&parsed, &rels.model_methods);
    
    // Combine
    let expanded = quote::quote! {
        #builder_code
        #factory_observer_code
        #model_code
    };
    
    TokenStream::from(expanded)
}
