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
            quote::quote! { self.#ident }
        })
        .collect::<Vec<_>>();

    let query = if let Some(soft_delete_column) = &args.soft_delete {
        format!(
            "UPDATE {table} SET {soft_delete_column} = NOW() WHERE {query_where}",
            query_where = query_where.join(" AND "),
        )
    } else {
        format!(
            "DELETE FROM {table} WHERE {query_where}",
            query_where = query_where.join(" AND "),
        )
    };

    let stream = quote::quote! {
        impl ::gremlin_orm::DeletableEntity for #ident {
            async fn delete<'a>(&self, executor: impl ::sqlx::PgExecutor<'a>) -> Result<(), sqlx::Error> {
                ::sqlx::query!(
                    #query,
                    #(#values_fields),*
                ).execute(executor).await?;

                Ok(())
            }
        }
    };

    stream
}
