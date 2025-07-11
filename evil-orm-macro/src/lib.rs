//! # `evil-orm`
#![expect(dead_code)]

use darling::{FromDeriveInput, FromField, ast::Data, util::Ignored};
use proc_macro::TokenStream;
use syn::{DeriveInput, Ident, parse_macro_input};
use thiserror::Error;

/// Generate the entity
#[proc_macro_derive(Entity, attributes(orm))]
pub fn derive_response(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);

    match generate(args) {
        Ok(stream) => stream,
        Err(err) => err.write_errors().into(),
    }
}

#[derive(Debug, Error)]
pub(crate) enum GeneratorError {
    #[error("{0}")]
    Syn(
        #[source]
        #[from]
        syn::Error,
    ),
    #[error("{0}")]
    Darling(
        #[source]
        #[from]
        darling::Error,
    ),
}

impl GeneratorError {
    pub(crate) fn write_errors(self) -> proc_macro2::TokenStream {
        match self {
            Self::Syn(err) => err.to_compile_error(),
            Self::Darling(err) => err.write_errors(),
        }
    }
}

fn generate(args: DeriveInput) -> Result<TokenStream, GeneratorError> {
    let args: EntityArgs = EntityArgs::from_derive_input(&args)?;

    let insert_stream = insert::generate_insert(&args);
    let update_stream = update::generate_update(&args);

    let stream = quote::quote! {
        #insert_stream
        #update_stream
    };

    Ok(stream.into())
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(orm), forward_attrs(doc))]
struct EntityArgs {
    ident: Ident,
    vis: syn::Visibility,
    data: Data<Ignored, EntityField>,
    table: String,
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(orm), forward_attrs(doc))]
struct EntityField {
    ident: Option<Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
    #[darling(default)]
    pk: bool,
    #[darling(default)]
    generated: bool,
    #[darling(default)]
    deref: bool,
}

mod insert {
    use proc_macro2::TokenStream;

    use crate::EntityArgs;

    pub(crate) fn generate_insert(args: &EntityArgs) -> TokenStream {
        let insertable_base = args
            .data
            .clone()
            .take_struct()
            .unwrap()
            .into_iter()
            .filter(|field| !field.generated)
            .collect::<Vec<_>>();

        let insertable_fields = insertable_base
            .iter()
            .cloned()
            .map(|field| {
                let ident = field.ident.unwrap();
                let vis = field.vis;
                let ty = field.ty;

                quote::quote! {
                    #vis #ident: #ty
                }
            })
            .collect::<Vec<_>>();

        let query_values = insertable_base
            .iter()
            .cloned()
            .map(|field| {
                let ident = field.ident.unwrap();

                if field.deref {
                    quote::quote! {
                        self.#ident.as_deref()
                    }
                } else {
                    quote::quote! {
                        self.#ident
                    }
                }
            })
            .collect::<Vec<_>>();

        let vis = args.vis.clone();

        let source_ident = args.ident.clone();
        let ident = quote::format_ident!("Insertable{}", args.ident);

        let table = args.table.clone();

        let query = format!(
            "INSERT INTO {table} ({fields}) VALUES ({values}) RETURNING *",
            fields = insertable_base
                .iter()
                .map(|field| field.ident.clone().unwrap().to_string())
                .collect::<Vec<_>>()
                .join(", "),
            values = insertable_base
                .iter()
                .enumerate()
                .map(|(idx, _)| format!("${}", idx + 1))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let stream = quote::quote! {
            #vis struct #ident {
                #(#insertable_fields),*
            }

            impl ::evil_orm::InsertableEntity for #ident {
                type SourceEntity = #source_ident;

                async fn insert(&self, pool: &::sqlx::PgPool) -> Result<Self::SourceEntity, ::sqlx::Error> {
                    ::sqlx::query_as!(
                        #source_ident,
                        #query,
                        #(#query_values),*
                    ).fetch_one(pool).await
                }
            }
        };

        stream
    }
}

mod update {
    use proc_macro2::TokenStream;

    use crate::EntityArgs;

    pub(crate) fn generate_update(args: &EntityArgs) -> TokenStream {
        let base = args
            .data
            .clone()
            .take_struct()
            .unwrap()
            .into_iter()
            .filter(|field| field.pk || !field.generated)
            .collect::<Vec<_>>();

        let entity_fields = base
            .iter()
            .cloned()
            .map(|field| {
                let ident = field.ident.unwrap();
                let vis = field.vis;
                let ty = field.ty;

                quote::quote! {
                    #vis #ident: #ty
                }
            })
            .collect::<Vec<_>>();

        let query_where = base
            .iter()
            .filter(|field| field.pk)
            .cloned()
            .enumerate()
            .map(|(idx, field)| {
                let ident = field.ident.unwrap();
                let idx = idx + 1;
                format!("{ident} = ${idx}")
            })
            .collect::<Vec<_>>();

        let query_set = base
            .iter()
            .filter(|field| !field.pk)
            .cloned()
            .enumerate()
            .map(|(idx, field)| {
                let ident = field.ident.unwrap();
                let idx = idx + query_where.len() + 1;
                format!("{ident} = ${idx}")
            })
            .collect::<Vec<_>>();

        let values_ids = base
            .iter()
            .filter(|field| field.pk)
            .cloned()
            .map(|field| {
                let ident = field.ident.unwrap();

                if field.deref {
                    quote::quote! {
                        self.#ident.as_deref()
                    }
                } else {
                    quote::quote! {
                        self.#ident
                    }
                }
            })
            .collect::<Vec<_>>();

        let values_fields = base
            .iter()
            .filter(|field| !field.pk)
            .cloned()
            .map(|field| {
                let ident = field.ident.unwrap();

                if field.deref {
                    quote::quote! {
                        self.#ident.as_deref()
                    }
                } else {
                    quote::quote! {
                        self.#ident
                    }
                }
            })
            .collect::<Vec<_>>();

        let vis = args.vis.clone();

        let source_ident = args.ident.clone();
        let ident = quote::format_ident!("Updatable{}", args.ident);

        let table = args.table.clone();

        let query = format!(
            "UPDATE {table} SET {query_set} WHERE {query_where} RETURNING *",
            query_where = query_where.join(" AND "),
            query_set = query_set.join(", "),
        );

        let stream = quote::quote! {
            #vis struct #ident {
                #(#entity_fields),*
            }

            impl ::evil_orm::UpdatableEntity for #ident {
                type SourceEntity = #source_ident;

                async fn update(&self, pool: &::sqlx::PgPool) -> Result<Self::SourceEntity, ::sqlx::Error> {
                    ::sqlx::query_as!(
                        #source_ident,
                        #query,
                        #(#values_ids),*,
                        #(#values_fields),*
                    ).fetch_one(pool).await
                }
            }
        };

        stream
    }
}
