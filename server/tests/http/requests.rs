use serde::Serialize;

#[derive(Serialize)]
pub struct CreateRecipe {
    pub title: String,
    pub image_id: Option<String>,
    pub ingredients: Vec<IngredientBlock>,
    pub instructions: Vec<InstructionBlock>,
    pub notes: Option<String>,
    pub tag_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct UpdateRecipe {
    pub previous_hash: String,
    pub title: String,
    pub image_id: Option<String>,
    pub ingredients: Vec<IngredientBlock>,
    pub instructions: Vec<InstructionBlock>,
    pub notes: Option<String>,
    pub tag_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct CreateTag {
    pub name: String,
}

#[derive(Serialize)]
pub struct IngredientBlock {
    pub title: Option<String>,
    pub ingredients: Vec<String>,
}

impl IngredientBlock {
    pub fn new(new: &[(Option<&str>, &[&str])]) -> Vec<Self> {
        new.iter()
            .map(|(t, i)| Self {
                title: t.map(|s| s.to_string()),
                ingredients: i.iter().map(|s| s.to_string()).collect(),
            })
            .collect()
    }
}

#[derive(Serialize)]
pub struct InstructionBlock {
    pub title: Option<String>,
    pub instructions: Vec<String>,
}

impl InstructionBlock {
    pub fn new(new: &[(Option<&str>, &[&str])]) -> Vec<Self> {
        new.iter()
            .map(|(t, i)| Self {
                title: t.map(|s| s.to_string()),
                instructions: i.iter().map(|s| s.to_string()).collect(),
            })
            .collect()
    }
}
