//! # `evil-orm`

pub use evil_orm_macro::Entity;

/// Trait for entities that can be inserted
pub trait InsertableEntity {
    /// Entity related to the insertable entity
    type SourceEntity;

    /// Insert the entity, returning either [`Self::SourceEntity`] or an error
    fn insert(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl Future<Output = Result<Self::SourceEntity, sqlx::Error>>;
}

/// Trait for entities that can be updated
pub trait UpdatableEntity {
    /// Entity related to the updatable entity
    type SourceEntity;

    /// Insert the entity, returning either [`Self::SourceEntity`] or an error
    fn update(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl Future<Output = Result<Self::SourceEntity, sqlx::Error>>;
}
