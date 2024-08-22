use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result, Row, Statement, ToSql};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tauri::State;
use r2d2::Pool;

use crate::configuration::Configuration;

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockContent {
    pub content_type: String,
    pub contents: Option<String>,
    pub title: Option<String>,
}

impl BlockContent {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn from_json_string(json_string: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_string)
    }

    pub fn new (block_content_type: String,raw_data: String) -> Self {
        BlockContent {
            content_type: block_content_type,
            contents: Some(raw_data),
            title: None,
        }
    }
}

#[derive (Debug, Deserialize, Serialize)]
pub struct Block {
    pub id: Uuid,
    pub parent_id: Uuid,
    pub block_type: String,
    pub block_contents: BlockContent,
    pub block_order: Option<i32>,
    pub favorite: bool,
    pub children: Vec<Block>,
    pub created_at_utc: DateTime<Utc>,
    pub updated_at_utc: DateTime<Utc>,
}

pub fn block_type_from_content_type(content_type: &str) -> String {
    match content_type {
        "paragraph" => "text".to_string(),
        "page" => "page".to_string(),
        _ => "text".to_string(),
    }
}

impl Block {
    pub fn new(
        parent_id: Uuid,
        block_content_type: String,
        raw_data: String,
    ) -> Self {
        Block {
            id: Uuid::now_v7(),
            parent_id: parent_id,
            block_type: block_type_from_content_type(block_content_type.as_str()), 
            block_contents: BlockContent::new(block_content_type, raw_data),
            block_order: None,
            favorite: false,
            children: Vec::new(),
            created_at_utc: Utc::now(),
            updated_at_utc: Utc::now(),
        }
    }

    pub fn save(&self, connection: &Connection) -> Result<(), ()> {
        log::debug!("Saving block {:?}", &self.id);

        if self.is_stored() {
            // TODO: Update block
            log::debug!("Block already stored {:?}", &self.id);
            return Ok(());
        }

        connection.execute(
            "INSERT INTO blocks (id, parent_id, block_type, data, block_order, favorite, created_at_utc, updated_at_utc) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                &self.id.to_string(), 
                &self.parent_id.to_string(), 
                &self.block_type, 
                &self.block_contents.to_json().unwrap(),
                match &self.load_last_block_in_same_page(connection).unwrap() {
                    Some(block) => block.block_order.unwrap_or_default() + 1,
                    None => 0,
                },
                &self.favorite,
                &self.created_at_utc.to_rfc3339(), 
                &self.updated_at_utc.to_rfc3339()],
        ).unwrap();

        Ok(())
    }

    fn load_last_block_in_same_page(
        &self,
        connection: &Connection,
    ) -> Result<Option<Block>, ()> {
        let mut statement: Statement;
        let mut rows;
        statement = connection.prepare("SELECT * FROM blocks WHERE parent_id = ?1 ORDER BY block_order DESC LIMIT 1").unwrap();
        rows = statement.query_map(rusqlite::params![self.parent_id.to_string()], |row| {
            Block::from_row(row, connection)
        }).unwrap();

        let mut blocks: Vec<Block> = Vec::new();
        for row in rows {
            blocks.push(row.unwrap());
        }

        return Ok(blocks.pop());
    }

    fn is_stored(&self) -> bool {
        false
    }

    fn from_row(row: &Row, connection: &Connection) -> Result<Self> {
        let uuid_string: String = row.get("id").unwrap();
        let parent_uuid_string: String = row.get("parent_id").unwrap();
        let created_at_string: String = row.get("created_at_utc").unwrap();
        let updated_at_string: String = row.get("updated_at_utc").unwrap();
        let block_data: String = row.get("data").unwrap();

        Ok(Block {
            id: Uuid::parse_str(&uuid_string).unwrap(),
            parent_id: Uuid::parse_str(&parent_uuid_string).unwrap(),
            block_type: row.get("block_type")?,
            block_contents: BlockContent::from_json_string(&block_data).unwrap(),
            block_order: row.get("block_order")?,
            favorite: row.get("favorite")?,
            children: Block::load_block_children_for_id(Uuid::parse_str(&uuid_string).unwrap(), connection).unwrap(),
            created_at_utc: DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&created_at_string).unwrap()),
            updated_at_utc: DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&&updated_at_string).unwrap())
        })
    }

    fn load_block_children_for_id(uuid: Uuid, connection: &Connection) -> Result<Vec<Block>, ()> {
        let mut statement: Statement;
        let mut rows;

        statement = connection.prepare("SELECT * FROM blocks WHERE parent_id = ?1 ORDER BY block_order ASC").unwrap();
        rows = statement.query_map(rusqlite::params![uuid.to_string()], |row| {
            Block::from_row(row, connection)
        }).unwrap();

        let mut blocks: Vec<Block> = Vec::new();
        for row in rows {
            blocks.push(row.unwrap());
        }

        Ok(blocks)
    }

    fn load_for_parent(
        parent_id: Uuid,
        connection: &Connection,
    ) -> Result<Vec<Block>, ()> {
        let mut statement: Statement;

        statement = connection.prepare("SELECT * FROM blocks WHERE parent_id = ?1 ORDER BY block_order ASC").unwrap();
        let rows = statement.query_map(rusqlite::params![parent_id.to_string()], |row| {
            Block::from_row(row, connection)
        }).unwrap();

        let mut blocks: Vec<Block> = Vec::new();
        for row in rows {
            blocks.push(row.unwrap());
        }

        Ok(blocks)
    }

    pub fn find_or_create_workspace_block(workspace_id: Uuid, connection: &Connection) -> Result<Self, String> {
        match Block::load_by_id(workspace_id, connection).unwrap() {
            Some(workspace_block) => return Ok(workspace_block),
            None => {
                let mut workspace_block = Block {
                    id: workspace_id,
                    parent_id: Uuid::now_v7(), // TODO: not sure what to do with the workspace parent, right now is just random uuid
                    block_type: "workspace".to_string(),
                    block_contents: BlockContent::new("workspace".to_string(), "".to_string()),
                    block_order: None,
                    favorite: false,
                    children: Vec::new(),
                    created_at_utc: Utc::now(),
                    updated_at_utc: Utc::now(),
                };

                let homepage = Block {
                    id: Uuid::now_v7(),
                    parent_id: workspace_id,
                    block_type: "page".to_string(),
                    block_contents: BlockContent::new("page".to_string(), "Home".to_string()),
                    block_order: None,
                    favorite: true,
                    children: Vec::new(),
                    created_at_utc: Utc::now(),
                    updated_at_utc: Utc::now(),
                };

                homepage.save(connection).unwrap();

                workspace_block.block_contents.contents.replace(String::from(&homepage.id.to_string()));
                workspace_block.children.push(homepage);
                workspace_block.save(connection).unwrap();

                Ok(workspace_block)
            }
        }
    }


    fn load_by_id(
        id: Uuid,
        connection: &Connection,
    ) -> Result<Option<Block>, ()> {
        let mut statement: Statement;
        let mut rows;

        statement = connection.prepare("SELECT * FROM blocks WHERE id = ?1").unwrap();
        rows = statement.query_map(rusqlite::params![id.to_string()], |row| {
            Block::from_row(row, connection)
        }).unwrap();

        let mut blocks: Vec<Block> = Vec::new();
        for row in rows {
            blocks.push(row.unwrap());
        }

        Ok(blocks.pop())
    }

    fn change_block_order(&self, new_order: i32, connection: &Connection) -> Result<Self, ()> {
        log::debug!("Moving block {} to position {}", self.id, new_order);


        if (new_order > self.block_order.unwrap_or_default()) {
            log::debug!("Moving other blocks up");
            let old_order = self.block_order.unwrap_or_default();            connection.execute(
                "UPDATE blocks SET block_order = (block_order - 1) WHERE parent_id = ?1 AND (block_order < ?2 OR block_order >= ?3)",
                rusqlite::params![
                    self.parent_id.to_string(),
                    old_order,
                    new_order
                ]
            ).unwrap();

        } else {
            log::debug!("Moving other blocks down");
            connection.execute(
                "UPDATE blocks SET block_order = block_order + 1 WHERE parent_id = ?1 AND block_order < ?2",
                rusqlite::params![self.parent_id.to_string(), self.block_order.unwrap_or_default()],
            ).unwrap();
        }

        connection.execute(
            "UPDATE blocks SET block_order = ?1 WHERE id = ?2",
            rusqlite::params![new_order, &self.id.to_string()],
        ).unwrap();

        Ok(Block::load_by_id(self.id, connection).unwrap().unwrap())
    }

}

#[tauri::command]
pub fn create_block_command(
    raw_data: String,
    block_type: String,
    parent_id: String,
    db: State<Pool<SqliteConnectionManager>>,
) -> Result<String, String> {
    log::debug!("Running create_block_command");
    let connection = db.get().expect("Could not get db connection");

    let block = Block::new(Uuid::parse_str(&parent_id).unwrap(), block_type, raw_data);
    block.save(&connection).unwrap();

    Ok(serde_json::to_string(&block).unwrap())
}

#[derive(Serialize)]
pub struct PageBlocksResponse {
    pub page_id: Option<String>,
    pub blocks: Vec<Block>,
}

#[tauri::command]
pub fn load_blocks_for_page_command(
    page_id: Option<String>,
    db: State<Pool<SqliteConnectionManager>>,
) -> Result<String, String> {
    log::debug!("Running load_blocks_for_page_command for page {:?}", page_id);
    let connection = db.get().expect("Could not get db connection");

    let response = PageBlocksResponse {
        page_id: page_id.clone(),
        blocks: Block::load_for_parent(Uuid::parse_str(&page_id.unwrap()).unwrap(), &connection).unwrap(),
    };

    Ok(serde_json::to_string(&response).unwrap())
}

#[tauri::command]
pub fn get_block_command(
    block_id: String,
    db: State<Pool<SqliteConnectionManager>>
) -> Result<String, String> {
    log::debug!("Running get_block_command for block {:?}", block_id);
    let connection = db.get().expect("Could not get db connection");

    let blocks = Block::load_by_id(Uuid::parse_str(&block_id).unwrap(), &connection).unwrap();

    Ok(serde_json::to_string(&blocks).unwrap())
}

#[tauri::command]
pub fn change_block_order_command(
    block_id: String,
    new_order: i32,
    db: State<Pool<SqliteConnectionManager>>,
) -> Result<String, String> {
    log::debug!("Running change_block_order_command for block {}", block_id);
    let connection = db.get().expect("Could not get db connection");

    let uuid = Uuid::parse_str(&block_id).unwrap();
    let block = Block::load_by_id(uuid, &connection).unwrap().unwrap();

    let result = block.change_block_order(new_order, &connection);

    match result {
        Ok(block) => Ok(serde_json::to_string(&block).unwrap()),
        Err(_) => Err("Could not change block order".to_string()),
    }
}

#[tauri::command]
pub fn load_home_page_command(
    db: State<Pool<SqliteConnectionManager>>,
    configuration: State<Configuration>
) -> Result<String, String> {
    log::debug!("Running load_home_page_command");
    let connection = db.get().expect("Could not get db connection");

    let mut block = Block::load_for_parent(configuration.workspace_id, &connection).unwrap();

    if block.len() == 0 {
        log::debug!("No home page block found, creating one.");

        let new_block = Block::new(configuration.workspace_id, "page".to_string(), "Home".to_string());
        new_block.save(&connection).unwrap();

        return Ok(serde_json::to_string(&new_block).unwrap());
    }
    log::debug!("Homepage loaded.");

    Ok(serde_json::to_string(&block.pop().unwrap()).unwrap())
}