# Gremlin ORM

A lightweight, type-safe ORM for PostgreSQL in Rust, built on top of SQLx with derive macro support for common CRUD operations.

## Features

- 🔒 **Type-safe** - Compile-time SQL query verification via SQLx
- 🚀 **Async/await** - Full async support with streaming capabilities
- 📝 **Derive macros** - Minimal boilerplate with `#[derive(Entity)]`
- 🐘 **PostgreSQL optimized** - Leverages PostgreSQL-specific features
- 🔄 **CRUD operations** - Insert, Update, Delete, and Stream entities
- 🏗️ **Generated fields** - Support for auto-increment IDs and computed columns

See the documentation on [docs.rs](https://docs.rs/gremlin-orm)

## Quick Start

Add `gremlin-orm` and `sqlx` to your `Cargo.toml`:

```toml
[dependencies]
sqlx = { version = "0.8.6", features = ["postgres", "runtime-tokio"] }
gremlin-orm = "0.1.0"
```

## Usage

### Define an Entity

```rust
use gremlin_orm::{Entity, InsertableEntity, UpdatableEntity, StreamableEntity, DeletableEntity};
use futures::StreamExt;

#[derive(Debug, Entity)]
#[orm(table = "public.users")]
struct User {
    #[orm(pk, generated)]
    id: i32,
    name: String,
    email: String,
    #[orm(generated)]
    created_at: chrono::DateTime<chrono::Utc>,
}
```

### Basic Operations

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = sqlx::PgPool::connect("postgresql://user:pass@localhost/db").await?;

    // Insert a new user
    let user = InsertableUser {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }
    .insert(&pool)
    .await?;

    println!("Created user: {:?}", user);

    // Update the user
    let mut updatable = UpdatableUser::from(user);
    updatable.name = "Alice Smith".to_string();
    let updated_user = updatable.update(&pool).await?;

    // Stream all users
    let users: Vec<_> = User::stream(&pool)
        .map(|result| result.unwrap())
        .collect()
        .await;

    // Delete the user
    updated_user.delete(&pool).await?;

    Ok(())
}
```

## License

This project is licensed under the GNU General Public License v3.0.
