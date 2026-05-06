use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::Movie;
use repository::MovieRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct MovieRow {
    movie_id:         String,
    title:            String,
    genre:            String,
    language:         String,
    duration_minutes: i32,
    poster_url:       Option<String>,
    rating:           f32,
    description:      String,
    created_at:       DateTime<Utc>,
}

impl From<MovieRow> for Movie {
    fn from(r: MovieRow) -> Self {
        Self {
            movie_id:         r.movie_id,
            title:            r.title,
            genre:            r.genre,
            language:         r.language,
            duration_minutes: r.duration_minutes as u32,
            poster_url:       r.poster_url,
            rating:           r.rating,
            description:      r.description,
            created_at:       r.created_at,
        }
    }
}

pub struct PgMovieRepository {
    pool: PgPool,
}

impl PgMovieRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl MovieRepository for PgMovieRepository {
    async fn save(&self, movie: Movie) -> Result<Movie, AppError> {
        let row = sqlx::query_as::<_, MovieRow>(
            r#"
            INSERT INTO movies (movie_id, title, genre, language, duration_minutes, poster_url, rating, description, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (movie_id) DO UPDATE SET
                title            = EXCLUDED.title,
                genre            = EXCLUDED.genre,
                language         = EXCLUDED.language,
                duration_minutes = EXCLUDED.duration_minutes,
                poster_url       = EXCLUDED.poster_url,
                rating           = EXCLUDED.rating,
                description      = EXCLUDED.description
            RETURNING *
            "#,
        )
        .bind(&movie.movie_id)
        .bind(&movie.title)
        .bind(&movie.genre)
        .bind(&movie.language)
        .bind(movie.duration_minutes as i32)
        .bind(&movie.poster_url)
        .bind(movie.rating)
        .bind(&movie.description)
        .bind(movie.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_id(&self, movie_id: &str) -> Result<Option<Movie>, AppError> {
        let row = sqlx::query_as::<_, MovieRow>("SELECT * FROM movies WHERE movie_id = $1")
            .bind(movie_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> Result<Vec<Movie>, AppError> {
        let rows = sqlx::query_as::<_, MovieRow>("SELECT * FROM movies ORDER BY created_at")
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete(&self, movie_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM movies WHERE movie_id = $1")
            .bind(movie_id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}
