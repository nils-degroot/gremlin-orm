//! # `gremlin-orm-macro`

use darling::{FromDeriveInput, FromField, ast::Data, util::Ignored};
use proc_macro::TokenStream;
use proc_macro_error2::abort;
use quote::ToTokens;
use syn::{DeriveInput, Ident, Path, parse_macro_input};
use thiserror::Error;

mod delete;
mod fetch;
mod insert;
mod stream;
mod update;

/// Generate the entity
#[proc_macro_error2::proc_macro_error]
#[proc_macro_derive(Entity, attributes(orm))]
pub fn derive_response(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);

    match generate(args) {
        Ok(stream) => stream,
        Err(err) => err.write_errors().into(),
    }
}

fn generate(args: DeriveInput) -> Result<TokenStream, GeneratorError> {
    let args: EntityArgs = EntityArgs::from_derive_input(&args)?;
    let ident = args.ident.clone();

    let args = match EntityCtx::try_from(args) {
        Ok(v) => v,
        Err(ParseCtxError::InvalidApplication) => {
            abort!(ident, ParseCtxError::InvalidApplication)
        }
    };

    let insert_stream = insert::generate_insert(&args);
    let update_stream = update::generate_update(&args);
    let stream_stream = stream::generate_stream(&args);
    let delete_stream = delete::generate_delete(&args);
    let get_by_id_stream = fetch::generate_fetch(&args);

    let stream = quote::quote! {
        #insert_stream
        #update_stream
        #stream_stream
        #delete_stream
        #get_by_id_stream
    };

    Ok(stream.into())
}

#[derive(Debug, Clone)]
struct EntityCtx {
    ident: Ident,
    vis: syn::Visibility,
    data: Vec<EntityFieldCtx>,
    table: String,
}

impl EntityCtx {
    fn pks(&self) -> impl Iterator<Item = &EntityFieldCtx> {
        self.data.iter().filter(|field| field.pk)
    }

    fn columns(&self) -> impl Iterator<Item = String> {
        self.data.iter().cloned().map(|field| {
            if let Some(cast) = field.cast {
                format!(
                    r#"{ident} AS "{ident}!: {cast}""#,
                    ident = field.ident,
                    cast = cast.to_token_stream()
                )
            } else {
                field.ident.to_string()
            }
        })
    }
}

impl TryFrom<EntityArgs> for EntityCtx {
    type Error = ParseCtxError;

    fn try_from(value: EntityArgs) -> Result<Self, Self::Error> {
        let mut data = vec![];

        for row in value
            .data
            .take_struct()
            .ok_or(ParseCtxError::InvalidApplication)?
        {
            data.push(row.try_into()?);
        }

        Ok(Self {
            ident: value.ident,
            vis: value.vis,
            data,
            table: value.table,
        })
    }
}

#[derive(Debug, Clone)]
struct EntityFieldCtx {
    ident: Ident,
    vis: syn::Visibility,
    ty: syn::Type,
    pk: bool,
    generated: bool,
    deref: bool,
    default: bool,
    cast: Option<Path>,
}

impl EntityFieldCtx {
    pub(crate) fn cast(&self) -> proc_macro2::TokenStream {
        self.cast
            .clone()
            .map(|cast| {
                quote::quote! {
                    as &#cast
                }
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Error)]
enum ParseCtxError {
    #[error("The `Entity` macro can only be applied to a struct with named fields")]
    InvalidApplication,
}

impl TryFrom<EntityField> for EntityFieldCtx {
    type Error = ParseCtxError;

    fn try_from(value: EntityField) -> Result<Self, Self::Error> {
        Ok(Self {
            ident: value.ident.ok_or(ParseCtxError::InvalidApplication)?,
            vis: value.vis,
            ty: value.ty,
            pk: value.pk,
            generated: value.generated,
            deref: value.deref,
            default: value.default,
            cast: value.cast,
        })
    }
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
    #[darling(default)]
    pk: bool,
    #[darling(default)]
    generated: bool,
    #[darling(default)]
    default: bool,
    #[darling(default)]
    deref: bool,
    cast: Option<syn::Path>,
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
