use std::collections::HashMap;

use entity::boards;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    database::board::{Mutation, Query},
    errors::AppError,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoardInput {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBoardInput {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardOutput {
    pub id: i32,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<boards::Model> for BoardOutput {
    fn from(model: boards::Model) -> Self {
        BoardOutput {
            id: model.id,
            name: model.name,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColumnExport {
    pub name: String,
    pub ordinal: i32,
    pub activities: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivityExport {
    pub name: String,
    pub body: Option<String>,
    pub ordinal: i32,
    pub tags: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColumnActivityExport {
    pub name: String,
    pub body: Option<String>,
    pub ordinal: i32,
    pub tags: Vec<i32>,
    pub column_id: i32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryExport {
    pub name: String,
    pub ordinal: i32,
    pub tags: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTagExport {
    pub name: String,
    pub color: String,
    pub ordinal: i32,
    pub category_id: i32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OtherTagExport {
    pub name: String,
    pub color: String,
    pub ordinal: i32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportBoardOutput {
    pub name: String,
    pub columns: HashMap<i32, ColumnExport>,
    pub activities: HashMap<i32, ColumnActivityExport>,
    pub other_activities: HashMap<i32, ActivityExport>,
    pub categories: HashMap<i32, CategoryExport>,
    pub category_tags: HashMap<i32, CategoryTagExport>,
    pub other_tags: HashMap<i32, OtherTagExport>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBoardInput {
    pub name: String,
    pub columns: HashMap<i32, ColumnExport>,
    pub activities: HashMap<i32, ColumnActivityExport>,
    pub other_activities: HashMap<i32, ActivityExport>,
    pub categories: HashMap<i32, CategoryExport>,
    pub category_tags: HashMap<i32, CategoryTagExport>,
    pub other_tags: HashMap<i32, OtherTagExport>,
}

#[tauri::command]
pub async fn get_all_boards(db: State<'_, DbConn>) -> Result<Vec<BoardOutput>, AppError> {
    let boards = Query::all_boards(db.inner()).await?;
    Ok(boards.into_iter().map(BoardOutput::from).collect())
}

#[tauri::command]
pub async fn get_board(db: State<'_, DbConn>, id: i32) -> Result<BoardOutput, AppError> {
    let board = Query::get_board_by_id(db.inner(), id).await?;
    Ok(BoardOutput::from(board))
}

#[tauri::command]
pub async fn create_board(
    db: State<'_, DbConn>,
    data: CreateBoardInput,
) -> Result<BoardOutput, AppError> {
    let board = Mutation::create_board(db.inner(), data).await?;
    Ok(BoardOutput::from(board))
}

#[tauri::command]
pub async fn update_board(
    db: State<'_, DbConn>,
    data: UpdateBoardInput,
) -> Result<BoardOutput, AppError> {
    let board = Mutation::update_board(db.inner(), data).await?;
    Ok(BoardOutput::from(board))
}

#[tauri::command]
pub async fn delete_board(db: State<'_, DbConn>, id: i32) -> Result<(), AppError> {
    Mutation::delete_board(db.inner(), id).await
}

#[tauri::command]
pub async fn duplicate_board(
    db: State<'_, DbConn>,
    source_id: i32,
    new_name: String,
) -> Result<BoardOutput, AppError> {
    let board = Mutation::duplicate_board(db.inner(), source_id, new_name).await?;
    Ok(BoardOutput::from(board))
}

#[tauri::command]
pub async fn export_board(
    db: State<'_, DbConn>,
    board_id: i32,
) -> Result<ExportBoardOutput, AppError> {
    Query::export_board(db.inner(), board_id).await
}

#[tauri::command]
pub async fn import_board(
    db: State<'_, DbConn>,
    data: ImportBoardInput,
) -> Result<BoardOutput, AppError> {
    let board = Mutation::import_board(db.inner(), data).await?;
    Ok(BoardOutput::from(board))
}
