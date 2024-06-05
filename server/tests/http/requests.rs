use serde::Serialize;

#[derive(Serialize)]
pub struct CreateRecipe {
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub notes: Option<String>,
}
