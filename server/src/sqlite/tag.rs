use std::collections::HashMap;

use rusqlite::{params, Connection};

use crate::{datastore::Error, domain};

pub fn get_tree(conn: &Connection) -> Result<domain::tag::Tree, Error> {
    let groups = get_groups(conn)?;
    let tags = get_tags(conn)?;

    let mut grouped: HashMap<Option<domain::tag::Id>, Vec<domain::tag::Tag>> = HashMap::new();
    for (tag, group_id) in tags {
        grouped.entry(group_id).or_insert_with(Vec::new).push(tag);
    }

    Ok(domain::tag::Tree {
        groups: groups
            .into_iter()
            .map(|group| {
                let id = group.id.clone();
                (group, grouped.remove(&Some(id)).unwrap_or(vec![]))
            })
            .collect(),
        ungrouped: grouped.remove(&None).unwrap_or(vec![]),
    })
}

pub fn get_all_with_stats(conn: &Connection) -> Result<Vec<domain::tag::WithStats>, Error> {
    let q = r#"
        SELECT tags.id, count(recipe_tags.recipe_id)
        FROM tags
        JOIN recipe_tags ON recipe_tags.tag_id = tags.id
        GROUP BY tags.id
        "#;

    let mut stmt = conn.prepare_cached(q)?;
    let result = stmt.query_and_then([], |row| -> Result<(domain::tag::Id, u64), Error> {
        let id: String = row.get(0)?;
        let count: u64 = row.get(1)?;

        Ok((domain::tag::Id::try_from(id.as_str())?, count))
    })?;
    let stats: Vec<(domain::tag::Id, u64)> = result.collect::<Vec<_>>()?;

    Ok(vec![])
}

pub fn insert(conn: &Connection, user_id: &str, name: &str) -> Result<domain::tag::Id, Error> {
    let id = domain::tag::Id::new();
    let id_string: String = id.clone().into();
    let mut stmt =
        conn.prepare_cached("INSERT INTO tags (id,name,created_by_user_id) VALUES (?1,?2,?3)")?;
    stmt.insert(params![id_string, name, user_id])?;

    Ok(id)
}

fn get_groups(conn: &Connection) -> Result<Vec<domain::tag::Group>, Error> {
    let q = "SELECT id,name,color,is_required,is_multi_select,description FROM tag_groups ORDER BY name ASC";

    let mut stmt = conn.prepare_cached(q)?;
    let result = stmt.query_and_then([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let color: String = row.get(2)?;
        let is_required: bool = row.get(3)?;
        let is_multi_select: bool = row.get(4)?;
        let description: Option<String> = row.get(5)?;

        Ok(domain::tag::Group {
            id: id.as_str().try_into()?,
            name: name.try_into()?,
            color: color.try_into()?,
            is_required,
            is_multi_select,
            description: match description {
                None => None,
                Some(s) => Some(s.try_into()?),
            },
        })
    })?;
    result.collect()
}

fn get_tags(conn: &Connection) -> Result<Vec<(domain::tag::Tag, Option<domain::tag::Id>)>, Error> {
    let q = "SELECT id,name,description,group_id FROM tags ORDER BY name ASC";

    let mut stmt = conn.prepare_cached(q)?;
    let result = stmt.query_and_then([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let description: Option<String> = row.get(2)?;
        let group_id: Option<String> = row.get(3)?;

        Ok((
            domain::tag::Tag {
                id: id.as_str().try_into()?,
                name: name.try_into()?,
                description: match description {
                    None => None,
                    Some(s) => Some(s.try_into()?),
                },
            },
            match group_id {
                None => None,
                Some(s) => Some(s.as_str().try_into()?),
            },
        ))
    })?;
    result.collect()
}
