use proc_macro2::TokenStream;

use crate::EntityCtx;

pub(crate) fn generate_stream(args: &EntityCtx) -> TokenStream {
    let ident = args.ident.clone();
    let table = args.table.clone();

    let query = format!(
        "SELECT {columns} FROM {table}",
        columns = args.columns().collect::<Vec<_>>().join(", ")
    );

    let stream = quote::quote! {
        impl ::gremlin_orm::StreamableEntity for #ident {
            fn stream(pool: &::sqlx::PgPool) -> impl ::gremlin_orm::Stream<Item = Result<Self, ::sqlx::Error>> {
                ::sqlx::query_as!(Self, #query).fetch(pool)
            }
        }
    };

    stream
}
