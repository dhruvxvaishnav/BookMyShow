use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::Venue;
use repository::VenueRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct VenueRow {
    venue_id: String,
    name: String,
    address: String,
    city: String,
    screen_count: i32,
    amenities: Vec<String>,
    created_at: DateTime<Utc>,
}

impl From<VenueRow> for Venue {
    fn from(r: VenueRow) -> Self {
        Self {
            venue_id: r.venue_id,
            name: r.name,
            address: r.address,
            city: r.city,
            screen_count: r.screen_count as u32,
            amenities: r.amenities,
            created_at: r.created_at,
        }
    }
}

pub struct PgVenueRepository {
    pool: PgPool,
}

impl PgVenueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VenueRepository for PgVenueRepository {
    async fn save(&self, venue: Venue) -> Result<Venue, AppError> {
        let row = sqlx::query_as::<_, VenueRow>(
            r#"
            INSERT INTO venues (venue_id, name, address, city, screen_count, amenities, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (venue_id) DO UPDATE SET
                name         = EXCLUDED.name,
                address      = EXCLUDED.address,
                city         = EXCLUDED.city,
                screen_count = EXCLUDED.screen_count,
                amenities    = EXCLUDED.amenities
            RETURNING *
            "#,
        )
        .bind(&venue.venue_id)
        .bind(&venue.name)
        .bind(&venue.address)
        .bind(&venue.city)
        .bind(venue.screen_count as i32)
        .bind(&venue.amenities)
        .bind(venue.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_id(&self, venue_id: &str) -> Result<Option<Venue>, AppError> {
        let row = sqlx::query_as::<_, VenueRow>("SELECT * FROM venues WHERE venue_id = $1")
            .bind(venue_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> Result<Vec<Venue>, AppError> {
        let rows = sqlx::query_as::<_, VenueRow>("SELECT * FROM venues ORDER BY created_at")
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_city(&self, city: &str) -> Result<Vec<Venue>, AppError> {
        let rows = sqlx::query_as::<_, VenueRow>(
            "SELECT * FROM venues WHERE LOWER(city) = LOWER($1) ORDER BY name",
        )
        .bind(city)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete(&self, venue_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM venues WHERE venue_id = $1")
            .bind(venue_id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}
