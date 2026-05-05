use async_trait::async_trait;
use common::AppError;
use domain::Show;
use repository::ShowRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryShowRepository {
    shows: RwLock<HashMap<String, Show>>,
}

impl InMemoryShowRepository {
    pub fn new() -> Self {
        Self {
            shows: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ShowRepository for InMemoryShowRepository {
    async fn save(&self, show: Show) -> Result<Show, AppError> {
        let mut w = self.shows.write().await;
        w.insert(show.show_id.clone(), show.clone());
        Ok(show)
    }

    async fn find_by_id(&self, show_id: &str) -> Result<Option<Show>, AppError> {
        let r = self.shows.read().await;
        Ok(r.get(show_id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<Show>, AppError> {
        let r = self.shows.read().await;
        Ok(r.values().cloned().collect())
    }

    async fn find_by_movie_id(&self, movie_id: &str) -> Result<Vec<Show>, AppError> {
        let r = self.shows.read().await;
        Ok(r.values()
            .filter(|s| s.movie_id.as_deref() == Some(movie_id))
            .cloned()
            .collect())
    }

    async fn exists(&self, show_id: &str) -> Result<bool, AppError> {
        let r = self.shows.read().await;
        Ok(r.contains_key(show_id))
    }

    async fn delete(&self, show_id: &str) -> Result<(), AppError> {
        let mut w = self.shows.write().await;
        w.remove(show_id);
        Ok(())
    }
}
