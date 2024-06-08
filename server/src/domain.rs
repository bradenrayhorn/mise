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
    pub ingredients: recipe::Ingredients,
    pub instructions: recipe::Instructions,
    pub notes: Option<recipe::Notes>,
    pub tag_ids: Vec<tag::Id>,
}

#[derive(Debug, Clone)]
pub struct UpdatingRecipe {
    pub id: recipe::Id,
    pub previous_hash: String,

    pub title: recipe::Title,
    pub ingredients: recipe::Ingredients,
    pub instructions: recipe::Instructions,
    pub notes: Option<recipe::Notes>,
    pub tag_ids: Vec<tag::Id>,
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: recipe::Id,
    pub hash: String,
    pub title: recipe::Title,
    pub ingredients: recipe::Ingredients,
    pub instructions: recipe::Instructions,
    pub notes: Option<recipe::Notes>,
    pub tags: Vec<tag::OnRecipe>,
}

#[derive(Debug, Clone)]
pub struct ListedRecipe {
    pub id: recipe::Id,
    pub title: recipe::Title,
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

    #[derive(Debug, Clone)]
    pub struct Ingredients {
        unparsed: String,
        blocks: Vec<IngredientBlock>,
    }

    #[derive(Debug, Clone)]
    pub struct IngredientBlock {
        title: Option<String>,
        ingredients: Vec<String>,
    }

    impl From<Ingredients> for String {
        fn from(value: Ingredients) -> Self {
            value.unparsed
        }
    }

    impl Ingredients {
        fn new(unparsed: &str) -> Result<Self, ValidationError> {
            let arena = comrak::Arena::new();
            let root = comrak::parse_document(&arena, unparsed, &comrak::Options::default());

            Ok(Ingredients {
                unparsed: unparsed.to_string(),
                blocks: match root.first_child() {
                    None => {
                        return Err(ValidationError::Constraint(
                            "ingredients is required.".into(),
                        ))
                    }
                    Some(first_node) => match first_node.data.borrow().value {
                        comrak::nodes::NodeValue::Heading(_) => {
                            let blocks = root
                                .children()
                                .step_by(2)
                                .zip(root.children().skip(1).step_by(2))
                                .map(
                                    |(heading, list)| -> Result<IngredientBlock, ValidationError> {
                                        Ok(IngredientBlock {
                                            title: Some(try_build_heading(heading)?),
                                            ingredients: try_build_list(list)?,
                                        })
                                    },
                                )
                                .collect::<Result<Vec<IngredientBlock>, ValidationError>>()?;
                            if blocks.is_empty() {
                                return Err(ValidationError::Format(
                                    "No ingredients found.".into(),
                                ));
                            }
                            blocks
                        }
                        comrak::nodes::NodeValue::List(_) => {
                            if root.children().count() > 1 {
                                return Err(ValidationError::Format(
                                    "Unknown items after list.".into(),
                                ));
                            }
                            vec![IngredientBlock {
                                title: None,
                                ingredients: gather_list(first_node),
                            }]
                        }
                        _ => {
                            return Err(ValidationError::Format(
                                "Only List or Heading must be at the top level.".into(),
                            ))
                        }
                    },
                },
            })
        }

        #[must_use]
        pub fn blocks(&self) -> &[IngredientBlock] {
            &self.blocks
        }
    }

    impl IngredientBlock {
        #[must_use]
        pub fn title(&self) -> Option<&str> {
            self.title.as_deref()
        }

        #[must_use]
        pub fn ingredients(&self) -> &[String] {
            &self.ingredients
        }
    }

    #[derive(Debug, Clone)]
    pub struct Instructions {
        unparsed: String,
        blocks: Vec<InstructionBlock>,
    }

    #[derive(Debug, Clone)]
    pub struct InstructionBlock {
        title: Option<String>,
        instructions: Vec<String>,
    }

    impl From<Instructions> for String {
        fn from(value: Instructions) -> Self {
            value.unparsed
        }
    }

    impl Instructions {
        fn new(unparsed: &str) -> Result<Self, ValidationError> {
            let arena = comrak::Arena::new();
            let root = comrak::parse_document(&arena, unparsed, &comrak::Options::default());

            Ok(Instructions {
                unparsed: unparsed.to_string(),
                blocks: match root.first_child() {
                    None => {
                        return Err(ValidationError::Constraint(
                            "instructions is required.".into(),
                        ))
                    }
                    Some(first_node) => match first_node.data.borrow().value {
                        comrak::nodes::NodeValue::Heading(_) => {
                            let blocks = root
                                .children()
                                .step_by(2)
                                .zip(root.children().skip(1).step_by(2))
                                .map(
                                    |(heading, list)| -> Result<InstructionBlock, ValidationError> {
                                        Ok(InstructionBlock {
                                            title: Some(try_build_heading(heading)?),
                                            instructions: try_build_list(list)?,
                                        })
                                    },
                                )
                                .collect::<Result<Vec<InstructionBlock>, ValidationError>>()?;
                            if blocks.is_empty() {
                                return Err(ValidationError::Format(
                                    "No instructions found.".into(),
                                ));
                            }
                            blocks
                        }
                        comrak::nodes::NodeValue::List(_) => {
                            if root.children().count() > 1 {
                                return Err(ValidationError::Format(
                                    "Unknown items after list.".into(),
                                ));
                            }
                            vec![InstructionBlock {
                                title: None,
                                instructions: gather_list(first_node),
                            }]
                        }
                        _ => {
                            return Err(ValidationError::Format(
                                "Only List or Heading must be at the top level.".into(),
                            ))
                        }
                    },
                },
            })
        }

        #[must_use]
        pub fn blocks(&self) -> &[InstructionBlock] {
            &self.blocks
        }
    }

    impl InstructionBlock {
        #[must_use]
        pub fn title(&self) -> Option<&str> {
            self.title.as_deref()
        }

        #[must_use]
        pub fn instructions(&self) -> &[String] {
            &self.instructions
        }
    }

    fn gather_text<'a>(node: &'a comrak::nodes::AstNode<'a>, string: &mut String) {
        match node.data.borrow().value {
            comrak::nodes::NodeValue::Text(ref text) => {
                string.push_str(text);
            }
            _ => {
                for child in node.children() {
                    gather_text(child, string);
                }
            }
        }
    }

    fn gather_list<'a>(node: &'a comrak::nodes::AstNode<'a>) -> Vec<String> {
        node.children()
            .filter_map(|child| {
                if let comrak::nodes::NodeValue::Item(_) = child.data.borrow().value {
                    let mut text = String::new();
                    gather_text(child, &mut text);
                    Some(text)
                } else {
                    None
                }
            })
            .collect()
    }

    fn try_build_heading<'a>(
        node: &'a comrak::nodes::AstNode<'a>,
    ) -> Result<String, ValidationError> {
        if let comrak::nodes::NodeValue::Heading(_) = node.data.borrow().value {
            let mut heading = String::new();
            gather_text(node, &mut heading);
            Ok(heading)
        } else {
            Err(ValidationError::Format(
                "Expected heading, found not a heading.".into(),
            ))
        }
    }

    fn try_build_list<'a>(
        node: &'a comrak::nodes::AstNode<'a>,
    ) -> Result<Vec<String>, ValidationError> {
        if let comrak::nodes::NodeValue::List(_) = node.data.borrow().value {
            Ok(gather_list(node))
        } else {
            Err(ValidationError::Format(
                "Expected list, found not a list.".into(),
            ))
        }
    }

    impl TryFrom<String> for Ingredients {
        type Error = ValidationError;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            let trimmed = value.trim();
            Self::new(trimmed)
        }
    }

    impl TryFrom<String> for Instructions {
        type Error = ValidationError;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            let trimmed = value.trim();
            Self::new(trimmed)
        }
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use self::recipe::IngredientBlock;

    use super::*;

    #[test]
    fn ingredients_with_just_a_list() -> Result<()> {
        let md = "
            * list item 1\n\
            * list item 2\n\
            * *list* _item_ 3
        "
        .to_string();
        let ingredients = recipe::Ingredients::try_from(md)?;
        let blocks = ingredients.blocks();
        assert_eq!(1, blocks.len());

        let block = blocks.first().unwrap();
        assert_block(
            block,
            None,
            &vec!["list item 1", "list item 2", "list item 3"],
        );

        Ok(())
    }

    #[test]
    fn ingredients_with_multiple_blocks() -> Result<()> {
        let md = "
            # For the Soup\n\
            * water\n\
            * spice\n\
            # For the Pancakes\n\
            * flour\n\
            * syrup
        "
        .to_string();
        let ingredients = recipe::Ingredients::try_from(md)?;
        let blocks = ingredients.blocks();
        assert_eq!(2, blocks.len());

        assert_block(&blocks[0], Some("For the Soup"), &vec!["water", "spice"]);
        assert_block(
            &blocks[1],
            Some("For the Pancakes"),
            &vec!["flour", "syrup"],
        );

        Ok(())
    }

    #[test]
    fn empty_ingredients() -> Result<()> {
        let md = "".to_string();
        let result = recipe::Ingredients::try_from(md);
        if let Err(ValidationError::Constraint(_)) = result {
        } else {
            panic!("Expected ValidationError::Constraint, got {result:?}");
        }

        Ok(())
    }

    #[test]
    fn ingredients_with_bad_top_level_item() -> Result<()> {
        let md = "
            > Block quote\
            "
        .to_string();
        let result = recipe::Ingredients::try_from(md);
        if let Err(ValidationError::Format(_)) = result {
        } else {
            panic!("Expected ValidationError::Format, got {result:?}");
        }

        Ok(())
    }

    #[test]
    fn ingredients_with_random_item_inside() -> Result<()> {
        let md = "
            - one\n\
            - two\n\
            > Block quote\n\
            - another\n\
            "
        .to_string();
        let result = recipe::Ingredients::try_from(md);
        if let Err(ValidationError::Format(_)) = result {
        } else {
            panic!("Expected ValidationError::Format, got {result:?}");
        }

        Ok(())
    }

    #[test]
    fn ingredients_mismatched_headings() -> Result<()> {
        let md = "
            # for the soup
            "
        .to_string();
        let result = recipe::Ingredients::try_from(md);
        if let Err(ValidationError::Format(_)) = result {
        } else {
            panic!("Expected ValidationError::Format, got {result:?}");
        }

        Ok(())
    }

    #[test]
    fn ingredients_mismatched_headings_and_list() -> Result<()> {
        let md = "
            # for the soup
            # for the chicken
            - item
            "
        .to_string();
        let result = recipe::Ingredients::try_from(md);
        if let Err(ValidationError::Format(_)) = result {
        } else {
            panic!("Expected ValidationError::Format, got {result:?}");
        }

        Ok(())
    }

    fn assert_block(block: &IngredientBlock, title: Option<&str>, ingredients: &[&str]) {
        assert_eq!(title, block.title());
        assert_eq!(ingredients, block.ingredients());
    }
}
