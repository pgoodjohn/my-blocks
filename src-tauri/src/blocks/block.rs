use serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result, Row, Statement, ToSql};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub enum BlockType {
    Text,
    Page,
}

impl Default for BlockType {
    fn default() -> Self {
        BlockType::Text
    }
}

pub struct BaseBlockContent {
    pub content_type: String,
}

pub struct TextBlockContent {
    pub base_content: BaseBlockContent,
}

pub struct PageBlockContent {
    pub base_content: BaseBlockContent,
}

pub trait BaseBlockTrait {
    fn get_content_type(&self) -> String;
}

impl BaseBlockTrait for TextBlockContent {
    fn get_content_type(&self) -> String {
        self.base_content.content_type.clone()
    }
}

impl BaseBlockTrait for PageBlockContent {
    fn get_content_type(&self) -> String {
        self.base_content.content_type.clone()
    }
}


pub struct Block {
    pub id: Uuid,
    pub parent_id: Uuid,
    pub block_type: String,
    pub block_contents: Box<dyn BaseBlockTrait>,
    pub block_order: Option<i32>,
    pub favorite: bool,
    pub children: Vec<Block>,
    pub created_at_utc: DateTime<Utc>,
    pub updated_at_utc: DateTime<Utc>,
}

impl Block {


    fn is_stored(&self) -> bool {
        false
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
}