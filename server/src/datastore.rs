use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use anyhow::anyhow;
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::oneshot;

use crate::domain::{
    self, Recipe, RecipeRevision, RegisteringUser, User, recipe::StringifiedBlock,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDocument {
    pub title: String,
    pub ingredients: Vec<StringifiedBlock>,
    pub instructions: Vec<StringifiedBlock>,
    pub notes: Option<String>,
    pub tag_ids: Vec<domain::tag::Id>,
    pub image_id: Option<domain::image::Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionedRecipeDocument {
    V1 {
        title: String,
        ingredients: Vec<StringifiedBlock>,
        instructions: Vec<StringifiedBlock>,
        notes: Option<String>,
        tag_ids: Vec<domain::tag::Id>,
        image_id: Option<domain::image::Id>,
    },
}

impl From<VersionedRecipeDocument> for RecipeDocument {
    fn from(value: VersionedRecipeDocument) -> Self {
        match value {
            VersionedRecipeDocument::V1 {
                title,
                ingredients,
                instructions,
                notes,
                tag_ids,
                image_id,
            } => RecipeDocument {
                title,
                ingredients,
                instructions,
                notes,
                tag_ids,
                image_id,
            },
        }
    }
}

impl From<RecipeDocument> for VersionedRecipeDocument {
    fn from(value: RecipeDocument) -> Self {
        VersionedRecipeDocument::V1 {
            title: value.title,
            ingredients: value.ingredients,
            instructions: value.instructions,
            notes: value.notes,
            tag_ids: value.tag_ids,
            image_id: value.image_id,
        }
    }
}

impl RecipeDocument {
    pub fn to_dumped_indexable_recipe(
        id: domain::recipe::Id,
        value: RecipeDocument,
    ) -> Result<domain::DumpedIndexableRecipe, domain::ValidationError> {
        Ok(domain::DumpedIndexableRecipe {
            id,
            title: value.title.try_into()?,
            ingredients: value
                .ingredients
                .into_iter()
                .map(domain::recipe::IngredientBlock::try_from)
                .collect::<Result<Vec<domain::recipe::IngredientBlock>, domain::ValidationError>>(
                )?,
            instructions: value
                .instructions
                .into_iter()
                .map(domain::recipe::InstructionBlock::try_from)
                .collect::<Result<Vec<domain::recipe::InstructionBlock>, domain::ValidationError>>(
                )?,
            notes: match value.notes {
                None => None,
                Some(s) => Some(s.try_into()?),
            },
            tag_ids: value.tag_ids,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HashedRecipeDocument {
    pub document: RecipeDocument,
    pub hash: String,
}

// Error

#[derive(Error, Debug)]
pub enum Error {
    #[error("record not found")]
    NotFound,

    #[error("invalid state when mutating record")]
    Conflict,

    #[error("no available connections")]
    NoConnections,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<domain::ValidationError> for Error {
    fn from(value: domain::ValidationError) -> Self {
        Error::Unknown(value.into())
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(value: tokio::sync::oneshot::error::RecvError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

// Pool

#[derive(Clone)]
pub struct Pool {
    // TODO - More efficient locking and data structure could be used, but this is fine for now.
    connections: Arc<Mutex<Vec<mpsc::Sender<Message>>>>,
}

#[derive(Clone)]
struct PoolConnection {
    sender: mpsc::Sender<Message>,
    connections: Arc<Mutex<Vec<mpsc::Sender<Message>>>>,
}

impl Drop for PoolConnection {
    fn drop(&mut self) {
        // release thread back to pool when connection is done
        let mut connections = self.connections.lock().unwrap();
        connections.push(self.sender.clone());
    }
}

impl Pool {
    #[must_use]
    pub fn new(connections: Vec<mpsc::Sender<Message>>) -> Self {
        Pool {
            connections: Arc::new(Mutex::new(connections)),
        }
    }

    fn maybe_get_conn(&self) -> Option<PoolConnection> {
        self.connections
            .lock()
            .unwrap()
            .pop()
            .map(|conn| PoolConnection {
                sender: conn,
                connections: self.connections.clone(),
            })
    }

    async fn conn(&self) -> Result<PoolConnection, Error> {
        // TODO - should check if the connection is healthy before use.
        // - benchmark the change before and after to make sure performance is still okay.
        //
        let mut retries = 0;
        while retries < 9 {
            let conn = self.maybe_get_conn();

            if let Some(conn) = conn {
                return Ok(conn);
            }

            let delay_ms = (2_u64.pow(retries) * 2) + (rand::rng().random_range(0..3));
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            retries += 1;
        }

        Err(Error::NoConnections)
    }

    pub async fn get_user(&self, id: String) -> Result<User, Error> {
        let conn = self.conn().await?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::GetUser { id, respond_to: tx };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn upsert_user_by_oauth_id(&self, user: RegisteringUser) -> Result<User, Error> {
        let conn = self.conn().await?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::UpsertUserByOauthId {
            user,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    // recipe

    pub async fn get_recipe(&self, id: String) -> Result<domain::Recipe, Error> {
        let conn = self.conn().await?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::GetRecipe { id, respond_to: tx };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn create_recipe(
        &self,
        id: String,
        user_id: String,
        recipe: RecipeDocument,
    ) -> Result<(), Error> {
        let conn = self.conn().await?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::CreateRecipe {
            id,
            user_id,
            recipe,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn update_recipe(
        &self,
        id: String,
        user_id: String,
        recipe: RecipeDocument,
        current_hash: String,
    ) -> Result<(), Error> {
        let conn = self.conn().await?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::UpdateRecipe {
            id,
            user_id,
            recipe,
            current_hash,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn list_recipes(
        &self,
        filter: domain::filter::Recipe,
        cursor: Option<domain::page::cursor::Recipe>,
    ) -> Result<domain::page::Recipe, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::ListRecipes {
            filter,
            cursor,
            respond_to: tx,
        };

        self.send_message(rx, msg).await
    }

    pub async fn get_recipe_revisions(
        &self,
        recipe_id: String,
    ) -> Result<Vec<RecipeRevision>, Error> {
        let conn = self.conn().await?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::GetRevisions {
            recipe_id,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn get_recipe_revision(
        &self,
        recipe_id: String,
        revision: usize,
    ) -> Result<domain::Recipe, Error> {
        let conn = self.conn().await?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::GetRevision {
            recipe_id,
            revision,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn dump_recipes_for_index(
        &self,
        cursor: Option<domain::page::cursor::DumpedIndexableRecipe>,
    ) -> Result<domain::page::DumpedIndexableRecipe, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::DumpRecipesForIndex {
            cursor,
            respond_to: tx,
        };

        self.send_message(rx, msg).await
    }

    // tags
    pub async fn create_tag(
        &self,
        user_id: String,
        name: String,
    ) -> Result<domain::tag::Id, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::CreateTag {
            user_id,
            name,
            respond_to: tx,
        };

        self.send_message(rx, msg).await
    }

    pub async fn get_tags(&self) -> Result<Vec<domain::tag::Tag>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::GetTags { respond_to: tx };

        self.send_message(rx, msg).await
    }

    // images
    pub async fn create_image(&self, id: &domain::image::Id) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::CreateImage {
            id: id.into(),
            respond_to: tx,
        };

        self.send_message(rx, msg).await
    }

    pub async fn get_image(&self, id: &domain::image::Id) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::GetImage {
            id: id.into(),
            respond_to: tx,
        };

        self.send_message(rx, msg).await
    }

    async fn send_message<T>(
        &self,
        rx: oneshot::Receiver<Result<T, Error>>,
        msg: Message,
    ) -> Result<T, Error> {
        let conn = self.conn().await?;
        let _ = conn.sender.send(msg);
        rx.await?
    }
}

pub enum Message {
    Health {
        respond_to: oneshot::Sender<Result<(), Error>>,
    },

    // user
    GetUser {
        id: String,
        respond_to: oneshot::Sender<Result<User, Error>>,
    },
    UpsertUserByOauthId {
        user: RegisteringUser,
        respond_to: oneshot::Sender<Result<User, Error>>,
    },

    // recipe
    GetRecipe {
        id: String,
        respond_to: oneshot::Sender<Result<Recipe, Error>>,
    },
    ListRecipes {
        filter: domain::filter::Recipe,
        cursor: Option<domain::page::cursor::Recipe>,
        respond_to: oneshot::Sender<Result<domain::page::Recipe, Error>>,
    },
    CreateRecipe {
        id: String,
        user_id: String,
        recipe: RecipeDocument,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    UpdateRecipe {
        id: String,
        user_id: String,
        recipe: RecipeDocument,
        current_hash: String,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    GetRevisions {
        recipe_id: String,
        respond_to: oneshot::Sender<Result<Vec<RecipeRevision>, Error>>,
    },
    GetRevision {
        recipe_id: String,
        revision: usize,
        respond_to: oneshot::Sender<Result<Recipe, Error>>,
    },
    DumpRecipesForIndex {
        cursor: Option<domain::page::cursor::DumpedIndexableRecipe>,
        respond_to: oneshot::Sender<Result<domain::page::DumpedIndexableRecipe, Error>>,
    },

    // tags
    GetTags {
        respond_to: oneshot::Sender<Result<Vec<domain::tag::Tag>, Error>>,
    },
    CreateTag {
        user_id: String,
        name: String,
        respond_to: oneshot::Sender<Result<domain::tag::Id, Error>>,
    },

    // images
    CreateImage {
        id: String,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    GetImage {
        id: String,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
}
