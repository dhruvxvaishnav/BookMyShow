#[derive(Debug, Clone)]
pub struct Show {
    pub show_id: String,
    pub show_name: String,
}

impl Show {
    pub fn new(show_id: String, show_name: String) -> Self {
        Show { show_id, show_name }
    }
}