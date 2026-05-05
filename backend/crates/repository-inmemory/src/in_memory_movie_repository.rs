use async_trait::async_trait;
use common::AppError;
use domain::Movie;
use repository::MovieRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryMovieRepository {
    movies: RwLock<HashMap<String, Movie>>,
}

impl InMemoryMovieRepository {
    pub fn new() -> Self {
        Self {
            movies: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MovieRepository for InMemoryMovieRepository {
    async fn save(&self, movie: Movie) -> Result<Movie, AppError> {
        let mut w = self.movies.write().await;
        w.insert(movie.movie_id.clone(), movie.clone());
        Ok(movie)
    }

    async fn find_by_id(&self, movie_id: &str) -> Result<Option<Movie>, AppError> {
        let r = self.movies.read().await;
        Ok(r.get(movie_id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<Movie>, AppError> {
        let r = self.movies.read().await;
        Ok(r.values().cloned().collect())
    }

    async fn delete(&self, movie_id: &str) -> Result<(), AppError> {
        let mut w = self.movies.write().await;
        w.remove(movie_id);
        Ok(())
    }
}
