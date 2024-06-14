use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Data<T> {
    pub data: T,
}

pub type CreateRecipe = Data<String>;
pub type GetRecipe = Data<Recipe>;
pub type CreateTag = Data<String>;
pub type GetTags = Data<Vec<Tag>>;
pub type CreateImage = Data<String>;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ListRecipes {
    pub next: Option<String>,
    pub data: Vec<ListedRecipe>,
}

// models

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Recipe {
    pub id: String,
    pub hash: String,
    pub title: String,
    pub image_id: Option<String>,
    pub ingredient_blocks: Vec<Ingredients>,
    pub instruction_blocks: Vec<Instructions>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ListedRecipe {
    pub id: String,
    pub title: String,
    pub image_id: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Instructions {
    pub title: Option<String>,
    pub instructions: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Ingredients {
    pub title: Option<String>,
    pub ingredients: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Tag {
    pub id: String,
    pub name: String,
}
