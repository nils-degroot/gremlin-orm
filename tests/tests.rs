use assert2::check;
use futures::StreamExt;
use gremlin_orm::{
    DeletableEntity, Entity, FetchableEntity, InsertableEntity, StreamableEntity, UpdatableEntity,
};
use sqlx::{PgPool, prelude::FromRow};

// Generic entity
#[derive(Debug, Entity, PartialEq, Eq, FromRow)]
#[orm(table = "public.artist")]
struct Artist {
    #[orm(pk, generated)]
    id: i32,
    name: String,
    #[orm(generated)]
    slug: String,
}

// Deref fields
#[derive(Debug, Entity, PartialEq, Eq, FromRow)]
#[orm(table = "public.release")]
struct Release {
    #[orm(pk, generated)]
    id: i32,
    name: String,
    #[orm(generated)]
    slug: String,
    #[orm(deref)]
    synonyms: Option<Vec<String>>,
}

// Only pk fields
#[derive(Debug, Entity, PartialEq, Eq, FromRow)]
#[orm(table = "public.artist_type")]
struct ArtistType {
    #[orm(pk)]
    name: String,
}

// Generated only fields
#[derive(Debug, Entity, PartialEq, Eq, FromRow)]
#[orm(table = "public.generated_only")]
struct GeneratedOnly {
    #[orm(pk, generated)]
    id: i32,
}

// Multiple pks
#[derive(Debug, Entity, PartialEq, Eq, FromRow)]
#[orm(table = "public.artist_release")]
struct ArtistRelease {
    #[orm(pk)]
    artist_id: i32,
    #[orm(pk)]
    release_id: i32,
}

// Defaultable fields
#[derive(Debug, Entity, PartialEq, Eq, FromRow)]
#[orm(table = "public.defaultable")]
struct Defaultable {
    #[orm(pk, generated)]
    id: i32,
    #[orm(default)]
    name: String,
}

mod insert {
    use gremlin_orm::Defaultable;

    use super::*;

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_insert_an_entity(pool: PgPool) {
        let artist = InsertableArtist {
            name: "Testings".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        check!(artist.name == "Testings".to_string());
        check!(artist.slug == "testings".to_string());

        let artist = ArtistPk::from(artist)
            .fetch(&pool)
            .await
            .expect("Failed to fetch artist")
            .expect("Could not find artist");

        check!(artist.name == "Testings".to_string());
        check!(artist.slug == "testings".to_string());
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_handle_deref_properly(pool: PgPool) {
        let release = InsertableRelease {
            name: "Testings".to_string(),
            synonyms: None,
        }
        .insert(&pool)
        .await
        .expect("Failed to insert release");

        check!(release.name == "Testings".to_string());

        let stored = ReleasePk::from(release)
            .fetch(&pool)
            .await
            .expect("Failed to fetch release")
            .expect("Could not find release");

        check!(stored.name == "Testings".to_string());
        check!(stored.synonyms == None);
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_handle_deref_properly_with_values(pool: PgPool) {
        let release = InsertableRelease {
            name: "Testings".to_string(),
            synonyms: Some(vec!["Release".to_string()]),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert release");

        check!(release.name == "Testings".to_string());

        let stored = ReleasePk::from(release)
            .fetch(&pool)
            .await
            .expect("Failed to fetch release")
            .expect("Could not find release");

        check!(stored.name == "Testings".to_string());
        check!(stored.synonyms == Some(vec!["Release".to_string()]));
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_generate_an_unit_struct_if_no_fields_can_be_inserted(pool: PgPool) {
        InsertableGeneratedOnly
            .insert(&pool)
            .await
            .expect("Failed to insert entity");

        let result = GeneratedOnly::stream(&pool).collect::<Vec<_>>().await;

        check!(result.len() == 1);
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_use_the_default_for_defaultable_default(pool: PgPool) {
        let entity = InsertableDefaultable {
            name: Defaultable::Default,
        }
        .insert(&pool)
        .await
        .expect("Failed to insert entity");

        check!(entity.name == "This is the default".to_string());
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_use_the_value_for_defaultable_value(pool: PgPool) {
        let entity = InsertableDefaultable {
            name: Defaultable::Value("Some name".to_string()),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert entity");

        check!(entity.name == "Some name".to_string());
    }
}

mod update {
    use super::*;

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_update_a_entity(pool: PgPool) {
        let artist = InsertableArtist {
            name: "Testings".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        let artist = UpdatableArtist {
            id: artist.id,
            name: "Updated".to_string(),
        }
        .update(&pool)
        .await
        .expect("Failed to update artist");

        check!(artist.name == "Updated".to_string());
        check!(artist.slug == "updated".to_string());

        let artist = ArtistPk::from(artist)
            .fetch(&pool)
            .await
            .expect("Failed to fetch artist")
            .expect("Could not find artist");

        check!(artist.name == "Updated".to_string());
        check!(artist.slug == "updated".to_string());
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_generate_a_from_implementation(pool: PgPool) {
        let artist = InsertableArtist {
            name: "Testings".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        let mut updatable = UpdatableArtist::from(artist);
        updatable.name = "Updated".to_string();

        let artist = updatable
            .update(&pool)
            .await
            .expect("Failed to update artist");

        check!(artist.name == "Updated".to_string());
        check!(artist.slug == "updated".to_string());

        let artist = ArtistPk::from(artist)
            .fetch(&pool)
            .await
            .expect("Failed to fetch artist")
            .expect("Could not find artist");

        check!(artist.name == "Updated".to_string());
        check!(artist.slug == "updated".to_string());
    }
}

mod delete {
    use super::*;

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_delete_the_entity(pool: PgPool) {
        let artist = InsertableArtist {
            name: "Testings".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        artist.delete(&pool).await.expect("Failed to delete artist");

        let result = sqlx::query_as!(Artist, "SELECT * FROM artist WHERE id = $1", artist.id)
            .fetch_optional(&pool)
            .await
            .expect("Failed to fetch artist");

        check!(let None = result);
    }
}

mod stream {
    use super::*;

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_list_all_entities(pool: PgPool) {
        let artist1 = InsertableArtist {
            name: "Testings 1".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        let artist2 = InsertableArtist {
            name: "Testings 2".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        let artists = Artist::stream(&pool)
            .map(|result| result.unwrap())
            .collect::<Vec<_>>()
            .await;

        assert_eq!(artists, vec![artist1, artist2])
    }
}

mod fetch {
    use super::*;

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_return_none_if_the_entity_does_not_exist(pool: PgPool) {
        let result = ArtistPk { id: 999 }
            .fetch(&pool)
            .await
            .expect("Failed to fetch artist");

        check!(let None = result);
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_return_the_entity_if_present(pool: PgPool) {
        let artist = InsertableArtist {
            name: "Testings".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        let artist = ArtistPk { id: artist.id }
            .fetch(&pool)
            .await
            .expect("Failed to fetch artist")
            .expect("Could not find artist");

        check!(artist.name == "Testings".to_string());
        check!(artist.slug == "testings".to_string());
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_generate_a_from_implementation_to_create_the_pk(pool: PgPool) {
        let artist = InsertableArtist {
            name: "Testings".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        let artist = ArtistPk::from(artist)
            .fetch(&pool)
            .await
            .expect("Failed to fetch artist")
            .expect("Could not find artist");

        check!(artist.name == "Testings".to_string());
        check!(artist.slug == "testings".to_string());
    }

    #[sqlx::test(fixtures("../resources/data/schema.sql"))]
    async fn it_should_use_multiple_fields_if_multiple_pks_are_set(pool: PgPool) {
        let artist = InsertableArtist {
            name: "Testings".to_string(),
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist");

        let release = InsertableRelease {
            name: "Testings".to_string(),
            synonyms: None,
        }
        .insert(&pool)
        .await
        .expect("Failed to insert release");

        let entity = InsertableArtistRelease {
            artist_id: artist.id,
            release_id: release.id,
        }
        .insert(&pool)
        .await
        .expect("Failed to insert artist_release");

        let pk = ArtistReleasePk::from(entity);

        check!(pk.artist_id == artist.id);
        check!(pk.release_id == release.id);

        let pk = pk
            .fetch(&pool)
            .await
            .expect("Failed to fetch artist_release")
            .expect("Could not find artist release");

        check!(pk.artist_id == artist.id);
        check!(pk.release_id == release.id);
    }
}
