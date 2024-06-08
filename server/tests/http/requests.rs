use serde::Serialize;

#[derive(Serialize)]
pub struct CreateRecipe {
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub notes: Option<String>,
    pub tag_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct UpdateRecipe {
    pub previous_hash: String,
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub notes: Option<String>,
    pub tag_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct CreateTag {
    pub name: String,
}
