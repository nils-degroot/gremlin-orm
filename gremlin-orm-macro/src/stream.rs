use proc_macro2::TokenStream;

use crate::EntityCtx;

pub(crate) fn generate_stream(args: &EntityCtx) -> TokenStream {
    let ident = args.ident.clone();
    let table = args.table.clone();

    let columns = args.columns().collect::<Vec<_>>().join(", ");

    let query = if let Some(soft_delete) = &args.soft_delete {
        format!("SELECT {columns} FROM {table} WHERE {soft_delete} IS NULL")
    } else {
        format!("SELECT {columns} FROM {table}")
    };

    let stream = quote::quote! {
        impl ::gremlin_orm::StreamableEntity for #ident {
            fn stream<'a>(executor: impl ::sqlx::PgExecutor<'a> + 'a) -> impl ::gremlin_orm::Stream<Item = Result<Self, ::sqlx::Error>> {
                ::sqlx::query_as!(Self, #query).fetch::<'_, 'a>(executor)
            }
        }
    };

    stream
}
