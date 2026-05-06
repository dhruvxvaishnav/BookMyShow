use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::Show;
use repository::ShowRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct ShowRow {
    show_id:        String,
    show_name:      String,
    theatre_name:   String,
    screen_number:  i32,
    start_time:     DateTime<Utc>,
    end_time:       DateTime<Utc>,
    price_per_seat: f64,
    total_seats:    i32,
    movie_id:       Option<String>,
    venue_id:       Option<String>,
    created_at:     DateTime<Utc>,
}

impl From<ShowRow> for Show {
    fn from(r: ShowRow) -> Self {
        Self {
            show_id:        r.show_id,
            show_name:      r.show_name,
            theatre_name:   r.theatre_name,
            screen_number:  r.screen_number as u32,
            start_time:     r.start_time,
            end_time:       r.end_time,
            price_per_seat: r.price_per_seat,
            total_seats:    r.total_seats as u32,
            movie_id:       r.movie_id,
            venue_id:       r.venue_id,
            created_at:     r.created_at,
        }
    }
}

pub struct PgShowRepository {
    pool: PgPool,
}

impl PgShowRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl ShowRepository for PgShowRepository {
    async fn save(&self, show: Show) -> Result<Show, AppError> {
        let row = sqlx::query_as::<_, ShowRow>(
            r#"
            INSERT INTO shows
                (show_id, show_name, theatre_name, screen_number, start_time, end_time,
                 price_per_seat, total_seats, movie_id, venue_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (show_id) DO UPDATE SET
                show_name      = EXCLUDED.show_name,
                theatre_name   = EXCLUDED.theatre_name,
                screen_number  = EXCLUDED.screen_number,
                start_time     = EXCLUDED.start_time,
                end_time       = EXCLUDED.end_time,
                price_per_seat = EXCLUDED.price_per_seat,
                total_seats    = EXCLUDED.total_seats,
                movie_id       = EXCLUDED.movie_id,
                venue_id       = EXCLUDED.venue_id
            RETURNING *
            "#,
        )
        .bind(&show.show_id)
        .bind(&show.show_name)
        .bind(&show.theatre_name)
        .bind(show.screen_number as i32)
        .bind(show.start_time)
        .bind(show.end_time)
        .bind(show.price_per_seat)
        .bind(show.total_seats as i32)
        .bind(&show.movie_id)
        .bind(&show.venue_id)
        .bind(show.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_id(&self, show_id: &str) -> Result<Option<Show>, AppError> {
        let row = sqlx::query_as::<_, ShowRow>("SELECT * FROM shows WHERE show_id = $1")
            .bind(show_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> Result<Vec<Show>, AppError> {
        let rows = sqlx::query_as::<_, ShowRow>("SELECT * FROM shows ORDER BY start_time")
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_movie_id(&self, movie_id: &str) -> Result<Vec<Show>, AppError> {
        let rows = sqlx::query_as::<_, ShowRow>(
            "SELECT * FROM shows WHERE movie_id = $1 ORDER BY start_time",
        )
        .bind(movie_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn exists(&self, show_id: &str) -> Result<bool, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM shows WHERE show_id = $1")
            .bind(show_id)
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(count.0 > 0)
    }

    async fn delete(&self, show_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM shows WHERE show_id = $1")
            .bind(show_id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}
