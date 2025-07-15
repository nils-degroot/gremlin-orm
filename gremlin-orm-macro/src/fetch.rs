use proc_macro2::TokenStream;

use crate::EntityCtx;

pub(crate) fn generate_fetch(args: &EntityCtx) -> TokenStream {
    let base = args.pks().cloned().collect::<Vec<_>>();

    let pk_fields = base
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

    let query_where = base
        .iter()
        .cloned()
        .enumerate()
        .map(|(idx, field)| {
            let ident = field.ident;
            let idx = idx + 1;
            format!("{ident} = ${idx}")
        })
        .collect::<Vec<_>>();

    let vis = args.vis.clone();

    let source_ident = args.ident.clone();
    let ident = quote::format_ident!("{}Pk", args.ident);

    let table = args.table.clone();

    let query = format!(
        "SELECT {columns} FROM {table} WHERE {query_where}",
        query_where = query_where.join(" AND "),
        columns = args.columns().collect::<Vec<_>>().join(", ")
    );

    let values_fields = base
        .iter()
        .cloned()
        .map(|field| {
            let ident = field.ident;

            quote::quote! {
                self.#ident
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

    let stream = quote::quote! {
        #vis struct #ident {
            #(#pk_fields),*
        }

        impl From<#source_ident> for #ident {
            fn from(value: #source_ident) -> Self {
                Self {
                    #(#from_fields),*
                }
            }
        }

        impl ::gremlin_orm::FetchableEntity for #ident {
            type SourceEntity = #source_ident;

            async fn fetch(&self, pool: &::sqlx::PgPool) -> Result<Option<Self::SourceEntity>, ::sqlx::Error> {
                ::sqlx::query_as!(
                    #source_ident,
                    #query,
                    #(#values_fields),*
                ).fetch_optional(pool).await
            }
        }
    };

    stream
}
