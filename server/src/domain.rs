use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub oauth_id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct RegisteringUser {
    pub potential_id: String,
    pub oauth_id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SessionKey(pub String);

impl Deref for SessionKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<SessionKey> for String {
    fn from(value: SessionKey) -> Self {
        value.0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub key: String,
    pub user_id: String,
    pub refresh_token: String,
    pub revalidate_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub enum SessionStatus {
    MustRevalidate,
    Expired,
    Ok,
}

impl Session {
    #[must_use]
    pub fn status(&self) -> SessionStatus {
        let now = chrono::Utc::now();

        if self.expires_at <= now {
            return SessionStatus::Expired;
        } else if self.revalidate_at <= now {
            return SessionStatus::MustRevalidate;
        }

        SessionStatus::Ok
    }
}

#[derive(Debug, Clone)]
pub struct RecipeRevision {
    pub revision: usize,
}

#[derive(Debug, Clone)]
pub struct CreatingRecipe {
    pub title: recipe::Title,
    pub image_id: Option<image::Id>,
    pub ingredients: Vec<recipe::IngredientBlock>,
    pub instructions: Vec<recipe::InstructionBlock>,
    pub notes: Option<recipe::Notes>,
    pub tag_ids: Vec<tag::Id>,
}

#[derive(Debug, Clone)]
pub struct UpdatingRecipe {
    pub id: recipe::Id,
    pub previous_hash: String,

    pub title: recipe::Title,
    pub image_id: Option<image::Id>,
    pub ingredients: Vec<recipe::IngredientBlock>,
    pub instructions: Vec<recipe::InstructionBlock>,
    pub notes: Option<recipe::Notes>,
    pub tag_ids: Vec<tag::Id>,
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: recipe::Id,
    pub hash: String,
    pub title: recipe::Title,
    pub image_id: Option<image::Id>,
    pub ingredients: Vec<recipe::IngredientBlock>,
    pub instructions: Vec<recipe::InstructionBlock>,
    pub notes: Option<recipe::Notes>,
    pub tags: Vec<tag::OnRecipe>,
}

#[derive(Debug, Clone)]
pub struct ListedRecipe {
    pub id: recipe::Id,
    pub title: recipe::Title,
    pub image_id: Option<image::Id>,
}

#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("{self}")]
    Constraint(String),
    #[error("{self}")]
    Format(String),
}

pub mod filter {

    #[derive(Debug, Clone)]
    pub struct Recipe {
        pub name: Option<String>,
        pub tag_ids: Vec<super::tag::Id>,
    }
}

pub mod page {
    use super::ListedRecipe;

    #[derive(Debug, Clone)]
    pub struct Recipe {
        pub items: Vec<ListedRecipe>,
        pub next: Option<cursor::Recipe>,
    }

    pub mod cursor {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Recipe {
            pub id: String,
            pub name: String,
        }
    }
}

mod id {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};

    use super::ValidationError;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Id {
        ulid: ulid::Ulid,
    }

    impl Id {
        #[must_use]
        pub fn new() -> Self {
            Id {
                ulid: ulid::Ulid::new(),
            }
        }
    }

    impl Serialize for Id {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            ulid::Ulid::serialize(&self.ulid, serializer)
        }
    }

    impl<'de> Deserialize<'de> for Id {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(Id {
                ulid: ulid::Ulid::deserialize(deserializer)?,
            })
        }
    }

    impl Default for Id {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Display for Id {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Display::fmt(&self.ulid, f)
        }
    }

    impl TryFrom<&str> for Id {
        type Error = ValidationError;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            Ok(Id {
                ulid: ulid::Ulid::from_string(value).map_err(|err| {
                    ValidationError::Format(format!("Could not parse ulid: {value}. {err}"))
                })?,
            })
        }
    }

    impl From<Id> for String {
        fn from(value: Id) -> Self {
            value.ulid.to_string()
        }
    }

    impl From<&Id> for String {
        fn from(value: &Id) -> Self {
            value.ulid.to_string()
        }
    }
}

pub mod user {
    #[derive(Debug, Clone)]
    pub struct Authenticated {
        pub id: String,
    }

    pub use super::id::Id;
}

pub mod recipe {

    use std::fmt::Debug;

    use serde::{Deserialize, Serialize};

    use crate::required_and_trimmed_string;

    use super::ValidationError;

    pub use super::id::Id;

    #[derive(Debug, Clone)]
    pub struct Title(String);

    impl TryFrom<String> for Title {
        type Error = ValidationError;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            let trimmed = value.trim();
            let char_count = trimmed.chars().count();
            if char_count < 1 {
                Err(ValidationError::Constraint(format!(
                    r#"Recipe title "{value}" must contain at least one character."#
                )))
            } else {
                Ok(Title(trimmed.to_string()))
            }
        }
    }

    impl From<Title> for String {
        fn from(value: Title) -> Self {
            value.0
        }
    }

    #[derive(Debug, Clone)]
    pub struct Notes(String);

    impl TryFrom<String> for Notes {
        type Error = ValidationError;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            let trimmed = value.trim();
            let char_count = trimmed.chars().count();
            if char_count < 1 {
                Err(ValidationError::Constraint(format!(
                    r#"Recipe notes "{value}" must contain at least one character, or else be null."#
                )))
            } else {
                Ok(Notes(trimmed.to_string()))
            }
        }
    }

    impl From<Notes> for String {
        fn from(value: Notes) -> Self {
            value.0
        }
    }

    impl Notes {
        #[must_use]
        pub fn into_html(&self) -> String {
            let mut unsafe_html = String::new();
            pulldown_cmark::html::push_html(&mut unsafe_html, pulldown_cmark::Parser::new(&self.0));

            ammonia::clean(&unsafe_html)
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct StringifiedBlock {
        pub title: Option<String>,
        pub items: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct IngredientBlock {
        pub title: Option<IngredientBlockTitle>,
        pub ingredients: Vec<Ingredient>,
    }

    impl TryFrom<StringifiedBlock> for IngredientBlock {
        type Error = ValidationError;
        fn try_from(value: StringifiedBlock) -> Result<Self, Self::Error> {
            Ok(Self {
                title: match value.title {
                    None => None,
                    Some(n) => Some(n.try_into()?),
                },
                ingredients: value
                    .items
                    .into_iter()
                    .map(Ingredient::try_from)
                    .collect::<Result<Vec<Ingredient>, ValidationError>>()?,
            })
        }
    }

    impl From<IngredientBlock> for StringifiedBlock {
        fn from(value: IngredientBlock) -> Self {
            StringifiedBlock {
                title: value.title.map(String::from),
                items: value.ingredients.into_iter().map(String::from).collect(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct InstructionBlock {
        pub title: Option<InstructionBlockTitle>,
        pub instructions: Vec<Instruction>,
    }

    impl TryFrom<StringifiedBlock> for InstructionBlock {
        type Error = ValidationError;
        fn try_from(value: StringifiedBlock) -> Result<Self, Self::Error> {
            Ok(Self {
                title: match value.title {
                    None => None,
                    Some(n) => Some(n.try_into()?),
                },
                instructions: value
                    .items
                    .into_iter()
                    .map(Instruction::try_from)
                    .collect::<Result<Vec<Instruction>, ValidationError>>()?,
            })
        }
    }

    impl From<InstructionBlock> for StringifiedBlock {
        fn from(value: InstructionBlock) -> Self {
            StringifiedBlock {
                title: value.title.map(String::from),
                items: value.instructions.into_iter().map(String::from).collect(),
            }
        }
    }

    required_and_trimmed_string!(InstructionBlockTitle);
    required_and_trimmed_string!(Instruction);

    impl Instruction {
        #[must_use]
        pub fn into_html(&self) -> String {
            let mut unsafe_html = String::new();
            pulldown_cmark::html::push_html(&mut unsafe_html, pulldown_cmark::Parser::new(&self.0));

            ammonia::clean(&unsafe_html)
        }
    }

    required_and_trimmed_string!(IngredientBlockTitle);
    required_and_trimmed_string!(Ingredient);
}

mod common {
    #[macro_export]
    macro_rules! required_and_trimmed_string {
        ($name:ident) => {
            #[derive(Debug, Clone)]
            pub struct $name(String);

            impl TryFrom<String> for $name {
                type Error = ValidationError;
                fn try_from(value: String) -> Result<Self, Self::Error> {
                    let trimmed = value.trim();
                    let char_count = trimmed.chars().count();
                    if char_count < 1 {
                        Err(ValidationError::Constraint(format!(
                            r#""{value}" must contain at least one character."#
                        )))
                    } else {
                        Ok($name(trimmed.to_string()))
                    }
                }
            }

            impl From<$name> for String {
                fn from(value: $name) -> Self {
                    value.0
                }
            }
        };
    }
}

pub mod tag {
    use super::ValidationError;

    pub use super::id::Id;

    #[derive(Debug, Clone)]
    pub struct Tag {
        pub id: Id,
        pub name: Name,
    }

    #[derive(Debug, Clone)]
    pub struct OnRecipe {
        pub id: Id,
        pub name: Name,
    }

    #[derive(Debug, Clone)]
    pub struct Creating {
        pub name: Name,
    }

    #[derive(Debug, Clone)]
    pub struct Name(String);

    impl TryFrom<String> for Name {
        type Error = ValidationError;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            let trimmed = value.trim();
            let char_count = trimmed.chars().count();
            if char_count < 1 {
                Err(ValidationError::Constraint(format!(
                    r#"Tag "{value}" must contain at least one character."#
                )))
            } else {
                Ok(Name(trimmed.to_string()))
            }
        }
    }

    impl From<Name> for String {
        fn from(value: Name) -> Self {
            value.0
        }
    }
}

pub mod image {
    pub use super::id::Id;
}
