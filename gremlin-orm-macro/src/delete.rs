use proc_macro2::TokenStream;

use crate::EntityCtx;

pub(crate) fn generate_delete(args: &EntityCtx) -> TokenStream {
    let ident = args.ident.clone();
    let table = args.table.clone();

    let base = args.pks().cloned().collect::<Vec<_>>();

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

    let query = format!(
        "DELETE FROM {table} WHERE {query_where}",
        query_where = query_where.join(" AND "),
    );

    let stream = quote::quote! {
        impl ::gremlin_orm::DeletableEntity for #ident {
            async fn delete(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
                ::sqlx::query!(
                    #query,
                    #(#values_fields),*
                ).execute(pool).await?;

                Ok(())
            }
        }
    };

    stream
}
