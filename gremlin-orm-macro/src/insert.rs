use proc_macro2::TokenStream;

use crate::EntityCtx;

pub(crate) fn generate_insert(args: &EntityCtx) -> TokenStream {
    let insertable_base = args
        .data
        .clone()
        .into_iter()
        .filter(|field| !field.generated)
        .collect::<Vec<_>>();

    let insertable_fields = insertable_base
        .iter()
        .cloned()
        .map(|field| {
            let ident = field.ident;
            let vis = field.vis;
            let ty = field.ty;

            if field.default {
                quote::quote! {
                    #vis #ident: ::gremlin_orm::Defaultable<#ty>
                }
            } else {
                quote::quote! {
                    #vis #ident: #ty
                }
            }
        })
        .collect::<Vec<_>>();

    let query_values = insertable_base
        .iter()
        .cloned()
        .map(|field| {
            let ident = field.ident;

            let deref = if field.deref {
                quote::quote! { .as_deref() }
            } else {
                TokenStream::default()
            };

            quote::quote! {
                self.#ident #deref
            }
        })
        .collect::<Vec<_>>();

    let vis = args.vis.clone();

    let source_ident = args.ident.clone();
    let ident = quote::format_ident!("Insertable{}", args.ident);

    let table = args.table.clone();

    // When no fields that could be inserted are present, use a simplified reprisentation
    if insertable_fields.is_empty() {
        let query = format!("INSERT INTO {table} DEFAULT VALUES RETURNING *");

        return quote::quote! {
            #vis struct #ident;

            impl ::gremlin_orm::InsertableEntity for #ident {
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
    }

    let mut static_field_names = vec![];
    let mut static_field_binds = vec![];

    for field in insertable_base.iter().filter(|field| !field.default) {
        let ident = field.ident.clone();

        static_field_names.push(ident.to_string());

        static_field_binds.push(quote::quote! {
            query = query.bind(&self.#ident);
        });
    }

    let mut optional_field_names = vec![];
    let mut optional_field_binds = vec![];

    for field in insertable_base.iter().filter(|field| field.default) {
        let ident = field.ident.clone();
        let ident_str = ident.to_string();

        optional_field_names.push(quote::quote! {
            if let ::gremlin_orm::Defaultable::Value(_) = &self.#ident {
                fields.push(#ident_str);
            }
        });

        optional_field_binds.push(quote::quote! {
            if let ::gremlin_orm::Defaultable::Value(v) = &self.#ident {
                query = query.bind(v);
            }
        });
    }

    let stream = quote::quote! {
        #vis struct #ident {
            #(#insertable_fields),*
        }

        impl ::gremlin_orm::InsertableEntity for #ident {
            type SourceEntity = #source_ident;

            async fn insert(&self, pool: &::sqlx::PgPool) -> Result<Self::SourceEntity, ::sqlx::Error> {
                let mut fields = vec![#(#static_field_names),*];
                #(#optional_field_names)*

                let table = #table;

                if fields.is_empty() {
                    let query = format!("INSERT INTO {table} DEFAULT VALUES RETURNING *");
                    ::sqlx::query_as::<_, Self::SourceEntity>(&query).fetch_one(pool).await
                } else {
                    let placeholders = (1..=fields.len())
                        .map(|i| format!("${}", i))
                        .collect::<Vec<_>>();

                    let query = format!(
                        "INSERT INTO {table} ({fields}) VALUES ({placeholders}) RETURNING *",
                        fields = fields.join(", "),
                        placeholders = placeholders.join(", ")
                    );

                    let mut query = ::sqlx::query_as::<_, Self::SourceEntity>(&query);
                    #(#static_field_binds)*
                    #(#optional_field_binds)*
                    query.fetch_one(pool).await
                }
            }
        }
    };

    stream
}
