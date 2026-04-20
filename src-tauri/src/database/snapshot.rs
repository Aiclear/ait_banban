use anyhow::Context;
use sea_orm::{DbConn, EntityTrait, QueryOrder, Set};

use ::entity::snapshots;
use crate::errors::AppError;

pub struct Query;

impl Query {
    pub async fn get_all_snapshots(db: &DbConn) -> Result<Vec<snapshots::Model>, AppError> {
        let snapshots = snapshots::Entity::find()
            .order_by_desc(snapshots::Column::CreatedAt)
            .all(db)
            .await?;
        Ok(snapshots)
    }

    pub async fn get_snapshot_by_id(db: &DbConn, id: i32) -> Result<Option<snapshots::Model>, AppError> {
        let snapshot = snapshots::Entity::find_by_id(id)
            .one(db)
            .await?;
        Ok(snapshot)
    }

    pub async fn get_snapshot_count(db: &DbConn) -> Result<u64, AppError> {
        let count = snapshots::Entity::find()
            .count(db)
            .await?;
        Ok(count)
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_snapshot(
        db: &DbConn,
        snapshot_data: String,
        created_at: String,
    ) -> Result<snapshots::Model, AppError> {
        let snapshot: snapshots::ActiveModel = snapshots::ActiveModel {
            snapshot_data: Set(snapshot_data),
            created_at: Set(created_at),
            ..Default::default()
        };

        let res = snapshot
            .insert(db)
            .await
            .context("failed to insert snapshot")?;

        Ok(res)
    }

    pub async fn delete_oldest_snapshots(db: &DbConn, keep_count: u64) -> Result<(), AppError> {
        let all_snapshots = snapshots::Entity::find()
            .order_by_asc(snapshots::Column::CreatedAt)
            .all(db)
            .await?;

        let total_count = all_snapshots.len() as u64;
        if total_count > keep_count {
            let to_delete = total_count - keep_count;
            for snapshot in all_snapshots.iter().take(to_delete as usize) {
                snapshots::Entity::delete_by_id(snapshot.id)
                    .exec(db)
                    .await
                    .context("failed to delete snapshot")?;
            }
        }

        Ok(())
    }

    pub async fn delete_snapshot_by_id(db: &DbConn, id: i32) -> Result<(), AppError> {
        snapshots::Entity::delete_by_id(id)
            .exec(db)
            .await
            .context("failed to delete snapshot")?;
        Ok(())
    }
}
