use std::{collections::HashSet, io::Cursor, thread};

use anyhow::{anyhow, Context};
use milli_v1::{
    documents::{DocumentsBatchBuilder, DocumentsBatchReader},
    update::{IndexDocuments, IndexDocumentsConfig, IndexerConfig, Settings},
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use crate::{datastore, domain};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Other(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    sender: mpsc::Sender<Message>,
}

impl Backend {
    pub fn new(index_path: &str, store: datastore::Pool) -> Result<Self, Error> {
        let sender = spawn_worker(index_path, store)?;

        Ok(Backend { sender })
    }

    pub async fn search(
        &self,
        query: String,
        filter: domain::filter::Recipe,
    ) -> Result<Vec<domain::recipe::Id>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Search {
            query,
            filter,
            respond_to: tx,
        };

        let _ = self.sender.send(msg).await;
        rx.await.context("SearchBackend::search")?
    }

    pub async fn index_recipes(&self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::IndexAll { respond_to: tx };

        let _ = self.sender.send(msg).await;
        rx.await.context("SearchBackend::index_recipes")?
    }
}

fn spawn_worker(index_path: &str, store: datastore::Pool) -> Result<mpsc::Sender<Message>, Error> {
    let (sender, mut receiver) = mpsc::channel(8);

    if !std::fs::exists(index_path)? {
        std::fs::create_dir_all(index_path)?;
    }
    let file_path = std::path::Path::new(index_path);

    let mut options = milli_v1::heed::EnvOpenOptions::new();
    options.map_size(50 * 1024 * 1024);

    let index = milli_v1::Index::new(options, file_path).context("open milli index")?;

    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                while let Some(msg) = receiver.recv().await {
                    match msg {
                        Message::Search {
                            respond_to,
                            query,
                            filter,
                        } => {
                            let _ = respond_to.send(search(&index, &query, filter));
                        }
                        Message::IndexAll { respond_to } => {
                            let _ = respond_to.send(load_recipes(&index, &store).await);
                        }
                    }
                }
            });
    });

    Ok(sender)
}

fn search(
    index: &milli_v1::Index,
    search: &str,
    filter: domain::filter::Recipe,
) -> Result<Vec<domain::recipe::Id>, Error> {
    let txn = index.read_txn().context("search: readtxn")?;

    let mut search_obj = milli_v1::Search::new(&txn, index);
    search_obj.query(search);
    search_obj.limit(20);

    let tag_id_strings: Vec<String> = filter.tag_ids.into_iter().map(String::from).collect();
    if !tag_id_strings.is_empty() {
        let cond = milli_v1::FilterCondition::And(
            tag_id_strings
                .iter()
                .map(|tag_id| milli_v1::FilterCondition::In {
                    fid: "tag_ids".into(),
                    els: vec![tag_id.as_ref()].into_iter().map(Into::into).collect(),
                })
                .collect(),
        );
        search_obj.filter(cond.into());
    }

    // get results
    let result = search_obj.execute().context("search: execute")?;
    let doc_ids = result.documents_ids;

    // convert to recipe ids
    let docs = index
        .iter_documents(&txn, doc_ids)
        .context("search: get documents")?;

    let fields_ids_map = index
        .fields_ids_map(&txn)
        .context("search: get field ids")?;
    let displayed_fields = index
        .displayed_fields_ids(&txn)
        .context("search: get displayed field ids")?
        .ok_or(anyhow!("search: missing displayed field ids"))?;

    let ids: Result<Vec<domain::recipe::Id>, Error> = docs
        .into_iter()
        .map(|doc| -> Result<domain::recipe::Id, Error> {
            let doc = doc.context("search: get document")?;
            let m = milli_v1::obkv_to_json(&displayed_fields, &fields_ids_map, doc.1)
                .context("search: parse document")?;

            let id = domain::recipe::Id::try_from(
                m.get("id")
                    .ok_or(anyhow!("search: missing id"))?
                    .as_str()
                    .ok_or(anyhow!("search: id is not string"))?,
            )
            .context("search: convert document id")?;

            Ok(id)
        })
        .collect();
    let ids = ids?;

    Ok(ids)
}

async fn load_recipes(index: &milli_v1::Index, store: &datastore::Pool) -> Result<(), Error> {
    let config = IndexerConfig::default();

    // update settings
    let mut wtxn = index.write_txn().context("index wtxn")?;
    let mut builder = Settings::new(&mut wtxn, index, &config);

    builder.set_autorize_typos(true);
    builder.set_min_word_len_one_typo(2);
    builder.set_min_word_len_two_typos(4);
    builder.set_searchable_fields(vec![
        "title".into(),
        "ingredients".into(),
        "instructions".into(),
    ]);
    let mut filterable_fields = HashSet::new();
    filterable_fields.insert("tag_ids".to_owned());
    builder.set_filterable_fields(filterable_fields);
    builder.set_displayed_fields(vec!["id".into()]);

    builder.execute(|_| (), || false).context("build")?;

    // index documents
    let indexing_config = IndexDocumentsConfig::default();
    let builder = IndexDocuments::new(&mut wtxn, index, &config, indexing_config, |_| (), || false)
        .context("index documents construct")?;

    let mut documents = DocumentsBatchBuilder::new(Vec::new());

    let mut cursor = None;
    loop {
        let page = store
            .dump_recipes_for_index(cursor)
            .await
            .context("dump recipes")?;

        for recipe in page.items {
            let mut obj = milli_v1::Object::new();

            obj.insert("id".into(), serde_json::Value::String(recipe.id.into()));
            obj.insert(
                "title".into(),
                serde_json::Value::String(recipe.title.into()),
            );
            obj.insert(
                "ingredients".into(),
                serde_json::Value::String(domain::recipe::dump_ingredient_block(
                    recipe.ingredients,
                )),
            );
            obj.insert(
                "instructions".into(),
                serde_json::Value::String(domain::recipe::dump_instruction_block(
                    recipe.instructions,
                )),
            );
            obj.insert(
                "tag_ids".into(),
                serde_json::Value::Array(
                    recipe
                        .tag_ids
                        .into_iter()
                        .map(|tag_id| serde_json::Value::String(tag_id.into()))
                        .collect(),
                ),
            );

            documents.append_json_object(&obj).context("add document")?;
        }

        if page.next.is_none() {
            break;
        }

        cursor = page.next;
    }

    // TODO - remove unwraps
    let vector = documents.into_inner().unwrap();

    let content = DocumentsBatchReader::from_reader(Cursor::new(vector)).unwrap();
    let (builder, user_error) = builder.add_documents(content).unwrap();
    user_error.unwrap();
    builder.execute().unwrap();

    wtxn.commit().unwrap();

    Ok(())
}

pub enum Message {
    IndexAll {
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    Search {
        query: String,
        filter: domain::filter::Recipe,
        respond_to: oneshot::Sender<Result<Vec<domain::recipe::Id>, Error>>,
    },
}
