use serde::Deserialize;

#[derive(Deserialize)]
pub struct Data<T> {
    pub data: T,
}

pub type Id = Data<uuid::Uuid>;

pub type GetRecipe = Data<Recipe>;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Recipe {
    pub id: String,
    pub hash: String,
    pub title: String,
    pub ingredient_blocks: Vec<Ingredients>,
    pub instruction_blocks: Vec<Instructions>,
    pub notes: Option<String>,
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
