use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{database::snapshot::{Mutation, Query}, errors::AppError};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
    pub id: i32,
    pub created_at: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotData {
    pub id: i32,
    pub created_at: String,
    pub snapshot_data: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSnapshotInput {
    pub snapshot_data: String,
    pub created_at: String,
}

const MAX_SNAPSHOTS: u64 = 30;

#[tauri::command]
pub async fn create_snapshot(
    db: State<'_, DbConn>,
    data: CreateSnapshotInput,
) -> Result<SnapshotInfo, AppError> {
    let snapshot = Mutation::create_snapshot(
        db.inner(),
        data.snapshot_data,
        data.created_at,
    ).await?;

    Mutation::delete_oldest_snapshots(db.inner(), MAX_SNAPSHOTS).await?;

    Ok(SnapshotInfo {
        id: snapshot.id,
        created_at: snapshot.created_at,
    })
}

#[tauri::command]
pub async fn get_all_snapshots(
    db: State<'_, DbConn>,
) -> Result<Vec<SnapshotInfo>, AppError> {
    let snapshots = Query::get_all_snapshots(db.inner()).await?;
    let result: Vec<SnapshotInfo> = snapshots
        .into_iter()
        .map(|s| SnapshotInfo {
            id: s.id,
            created_at: s.created_at,
        })
        .collect();
    Ok(result)
}

#[tauri::command]
pub async fn get_snapshot(
    db: State<'_, DbConn>,
    id: i32,
) -> Result<Option<SnapshotData>, AppError> {
    let snapshot = Query::get_snapshot_by_id(db.inner(), id).await?;
    Ok(snapshot.map(|s| SnapshotData {
        id: s.id,
        created_at: s.created_at,
        snapshot_data: s.snapshot_data,
    }))
}

#[tauri::command]
pub async fn delete_snapshot(
    db: State<'_, DbConn>,
    id: i32,
) -> Result<(), AppError> {
    Mutation::delete_snapshot_by_id(db.inner(), id).await
}

#[tauri::command]
pub async fn get_snapshot_count(
    db: State<'_, DbConn>,
) -> Result<u64, AppError> {
    let count = Query::get_snapshot_count(db.inner()).await?;
    Ok(count)
}
