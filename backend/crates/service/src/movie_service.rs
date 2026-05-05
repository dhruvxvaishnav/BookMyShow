use common::AppError;
use domain::{Movie, Show};
use repository::{MovieRepository, ShowRepository};
use std::sync::Arc;
use uuid::Uuid;

pub struct MovieService {
    pub movie_repo: Arc<dyn MovieRepository>,
    pub show_repo: Arc<dyn ShowRepository>,
}

impl MovieService {
    pub fn new(movie_repo: Arc<dyn MovieRepository>, show_repo: Arc<dyn ShowRepository>) -> Self {
        Self {
            movie_repo,
            show_repo,
        }
    }

    pub async fn create_movie(
        &self,
        title: String,
        genre: String,
        language: String,
        duration_minutes: u32,
        poster_url: Option<String>,
        rating: f32,
        description: String,
    ) -> Result<Movie, AppError> {
        let movie = Movie::new(
            Uuid::new_v4().to_string(),
            title,
            genre,
            language,
            duration_minutes,
            poster_url,
            rating,
            description,
        );
        self.movie_repo.save(movie).await
    }

    pub async fn list_movies(&self) -> Result<Vec<Movie>, AppError> {
        let mut movies = self.movie_repo.find_all().await?;
        movies.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(movies)
    }

    pub async fn get_movie(&self, movie_id: &str) -> Result<Option<Movie>, AppError> {
        self.movie_repo.find_by_id(movie_id).await
    }

    pub async fn list_shows_for_movie(&self, movie_id: &str) -> Result<Vec<Show>, AppError> {
        self.movie_repo
            .find_by_id(movie_id)
            .await?
            .ok_or_else(|| AppError::MovieNotFound(movie_id.to_string()))?;
        self.show_repo.find_by_movie_id(movie_id).await
    }
}
