use proc_macro2::TokenStream;

use crate::EntityCtx;

pub(crate) fn generate_update(args: &EntityCtx) -> TokenStream {
    let base = args
        .data
        .clone()
        .into_iter()
        .filter(|field| field.pk || !field.generated)
        .collect::<Vec<_>>();

    let entity_fields = base
        .iter()
        .cloned()
        .map(|field| {
            let ident = field.ident;
            let vis = field.vis;
            let ty = field.ty;

            quote::quote! {
                #vis #ident: #ty
            }
        })
        .collect::<Vec<_>>();

    let from_fields = base
        .iter()
        .cloned()
        .map(|field| {
            let ident = field.ident;

            quote::quote! {
                #ident: value.#ident
            }
        })
        .collect::<Vec<_>>();

    let query_where = base
        .iter()
        .filter(|field| field.pk)
        .cloned()
        .enumerate()
        .map(|(idx, field)| {
            let ident = field.ident;
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
            let ident = field.ident;
            let idx = idx + query_where.len() + 1;
            format!("{ident} = ${idx}")
        })
        .collect::<Vec<_>>();

    let values_ids = base
        .iter()
        .filter(|field| field.pk)
        .cloned()
        .map(|field| {
            let ident = field.ident;

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
            let ident = field.ident;

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

    // When no fields are updatable, we should just skip the whole generation step, so just return
    // an empty stream here
    if values_fields.is_empty() {
        return TokenStream::default();
    }

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

        impl From<#source_ident> for #ident {
            fn from(value: #source_ident) -> Self {
                Self {
                    #(#from_fields),*
                }
            }
        }

        impl ::gremlin_orm::UpdatableEntity for #ident {
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
