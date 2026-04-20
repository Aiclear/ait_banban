use std::collections::HashMap;

use crate::{
    commands::board::{
        CreateBoardInput, ExportBoardOutput, ImportBoardInput, UpdateBoardInput,
    },
    errors::AppError,
    utils::coloring::string_to_rgb_int,
};
use anyhow::Context;
use entity::{
    activities, boards,
    boards::{Entity as Board, Model as BoardModel},
    categories, category_tags, columns,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbConn, EntityTrait, IntoActiveModel,
    QueryFilter, Set, TransactionTrait,
};

pub struct Query;

impl Query {
    /// Fetches all boards.
    pub async fn all_boards(db: &DbConn) -> Result<Vec<BoardModel>, AppError> {
        let res = Board::find()
            .all(db)
            .await
            .context("failed to fetch all boards")?;
        Ok(res)
    }

    /// Fetches a board by id.
    pub async fn get_board_by_id(db: &DbConn, id: i32) -> Result<BoardModel, AppError> {
        let res = Board::find_by_id(id)
            .one(db)
            .await
            .context("failed to fetch board by id")?
            .ok_or(AppError::RowNotFound)?;
        Ok(res)
    }

    /// Exports a board's data as JSON.
    pub async fn export_board(db: &DbConn, board_id: i32) -> Result<ExportBoardOutput, AppError> {
        let board = Self::get_board_by_id(db, board_id).await?;

        let columns_data = columns::Entity::find()
            .find_with_related(activities::Entity)
            .filter(columns::Column::BoardId.eq(board_id))
            .all(db)
            .await
            .context("failed to fetch columns for export")?;

        let categories_data = categories::Entity::find()
            .find_with_related(category_tags::Entity)
            .filter(categories::Column::BoardId.eq(board_id))
            .all(db)
            .await
            .context("failed to fetch categories for export")?;

        let other_activities = activities::Entity::find()
            .filter(activities::Column::ColumnId.is_null())
            .all(db)
            .await
            .context("failed to fetch other activities for export")?;

        let mut columns = HashMap::new();
        let mut activities = HashMap::new();
        let mut other_activities_map = HashMap::new();

        for (column, col_activities) in columns_data {
            let activity_ids: Vec<i32> = col_activities.iter().map(|a| a.id).collect();
            columns.insert(
                column.id,
                crate::commands::fetch::ColumnOutput {
                    name: column.name,
                    ordinal: column.ordinal,
                    activities: activity_ids,
                },
            );

            for activity in col_activities {
                activities.insert(
                    activity.id,
                    crate::commands::fetch::ColumnActivityOutput {
                        name: activity.name,
                        body: activity.body,
                        ordinal: activity.ordinal,
                        tags: vec![],
                        column_id: activity.column_id.unwrap_or(0),
                    },
                );
            }
        }

        for activity in other_activities {
            other_activities_map.insert(
                activity.id,
                crate::commands::fetch::ActivityOutput {
                    name: activity.name,
                    body: activity.body,
                    ordinal: activity.ordinal,
                    tags: vec![],
                },
            );
        }

        let mut categories = HashMap::new();
        let mut category_tags_map = HashMap::new();

        for (category, tags) in categories_data {
            let tag_ids: Vec<i32> = tags.iter().map(|t| t.id).collect();
            categories.insert(
                category.id,
                crate::commands::fetch::CategoryOutput {
                    name: category.name,
                    ordinal: category.ordinal,
                    tags: tag_ids,
                },
            );

            for tag in tags {
                category_tags_map.insert(
                    tag.id,
                    crate::commands::fetch::CategoryTagOutput {
                        name: tag.tag_name,
                        color: crate::utils::coloring::rgb_int_to_string(tag.color),
                        ordinal: tag.ordinal,
                        category_id: category.id,
                    },
                );
            }
        }

        Ok(ExportBoardOutput {
            name: board.name,
            columns,
            activities,
            other_activities: other_activities_map,
            categories,
            category_tags: category_tags_map,
            other_tags: HashMap::new(),
        })
    }
}

pub struct Mutation;

impl Mutation {
    /// Creates a new board.
    pub async fn create_board(db: &DbConn, data: CreateBoardInput) -> Result<BoardModel, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let model = boards::ActiveModel {
            name: Set(data.name),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            ..Default::default()
        };
        let model = model.insert(db).await.context("failed to create board")?;
        Ok(model)
    }

    /// Updates a board.
    pub async fn update_board(db: &DbConn, data: UpdateBoardInput) -> Result<BoardModel, AppError> {
        let mut model = Board::find_by_id(data.id)
            .one(db)
            .await
            .context("failed to fetch board")?
            .ok_or(AppError::RowNotFound)?
            .into_active_model();

        if let Some(name) = data.name {
            model.name = Set(name);
        }
        model.updated_at = Set(chrono::Utc::now().to_rfc3339());

        let model = model.update(db).await.context("failed to update board")?;
        Ok(model)
    }

    /// Deletes a board.
    pub async fn delete_board(db: &DbConn, board_id: i32) -> Result<(), AppError> {
        let tr = db.begin().await.context("failed to begin transaction")?;

        activities::Entity::delete_many()
            .filter(
                activities::Column::ColumnId.in_subquery(
                    columns::Entity::find()
                        .select_only()
                        .column(columns::Column::Id)
                        .filter(columns::Column::BoardId.eq(board_id))
                        .into_query(),
                ),
            )
            .exec(&tr)
            .await
            .context("failed to delete activities")?;

        category_tags::Entity::delete_many()
            .filter(
                category_tags::Column::CategoryId.in_subquery(
                    categories::Entity::find()
                        .select_only()
                        .column(categories::Column::Id)
                        .filter(categories::Column::BoardId.eq(board_id))
                        .into_query(),
                ),
            )
            .exec(&tr)
            .await
            .context("failed to delete category tags")?;

        columns::Entity::delete_many()
            .filter(columns::Column::BoardId.eq(board_id))
            .exec(&tr)
            .await
            .context("failed to delete columns")?;

        categories::Entity::delete_many()
            .filter(categories::Column::BoardId.eq(board_id))
            .exec(&tr)
            .await
            .context("failed to delete categories")?;

        Board::delete_by_id(board_id)
            .exec(&tr)
            .await
            .context("failed to delete board")?;

        tr.commit().await.context("failed to commit transaction")?;

        Ok(())
    }

    /// Duplicates a board.
    pub async fn duplicate_board(db: &DbConn, source_board_id: i32, new_name: String) -> Result<BoardModel, AppError> {
        let exported = Query::export_board(db, source_board_id).await?;

        let new_board = Self::create_board(
            db,
            CreateBoardInput {
                name: if new_name.is_empty() {
                    format!("{} (Copy)", exported.name)
                } else {
                    new_name
                },
            },
        )
        .await?;

        let tr = db.begin().await.context("failed to begin transaction")?;

        let mut column_id_map: HashMap<i32, i32> = HashMap::new();
        let mut category_id_map: HashMap<i32, i32> = HashMap::new();
        let mut tag_id_map: HashMap<i32, i32> = HashMap::new();

        for (_, category) in exported.categories {
            let model = categories::ActiveModel {
                name: Set(category.name),
                ordinal: Set(category.ordinal),
                board_id: Set(Some(new_board.id)),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert category")?;
            category_id_map.insert(
                exported
                    .categories
                    .iter()
                    .find(|(_, c)| c.name == category.name && c.ordinal == category.ordinal)
                    .map(|(id, _)| *id)
                    .unwrap_or(0),
                inserted.id,
            );
        }

        for (tag_id, tag) in exported.category_tags {
            let new_category_id = category_id_map.get(&tag.category_id).copied();
            let model = category_tags::ActiveModel {
                tag_name: Set(tag.name),
                color: Set(string_to_rgb_int(&tag.color)),
                ordinal: Set(tag.ordinal),
                category_id: Set(new_category_id),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert tag")?;
            tag_id_map.insert(*tag_id, inserted.id);
        }

        for (_, column) in exported.columns {
            let model = columns::ActiveModel {
                name: Set(column.name),
                ordinal: Set(column.ordinal),
                board_id: Set(Some(new_board.id)),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert column")?;
            column_id_map.insert(
                exported
                    .columns
                    .iter()
                    .find(|(_, c)| c.name == column.name && c.ordinal == column.ordinal)
                    .map(|(id, _)| *id)
                    .unwrap_or(0),
                inserted.id,
            );
        }

        for (_, activity) in exported.activities {
            let new_column_id = activity.column_id.and_then(|id| column_id_map.get(&id).copied());
            let model = activities::ActiveModel {
                name: Set(activity.name),
                body: Set(activity.body),
                column_id: Set(new_column_id),
                ordinal: Set(activity.ordinal),
                ..Default::default()
            };
            model.insert(&tr).await.context("failed to insert activity")?;
        }

        for (_, activity) in exported.other_activities {
            let model = activities::ActiveModel {
                name: Set(activity.name),
                body: Set(activity.body),
                column_id: Set(None),
                ordinal: Set(activity.ordinal),
                ..Default::default()
            };
            model.insert(&tr).await.context("failed to insert other activity")?;
        }

        tr.commit().await.context("failed to commit transaction")?;

        Ok(new_board)
    }

    /// Imports a board from JSON.
    pub async fn import_board(db: &DbConn, data: ImportBoardInput) -> Result<BoardModel, AppError> {
        let new_board = Self::create_board(
            db,
            CreateBoardInput {
                name: if data.name.is_empty() {
                    "Imported Board".to_string()
                } else {
                    data.name
                },
            },
        )
        .await?;

        let tr = db.begin().await.context("failed to begin transaction")?;

        let mut column_id_map: HashMap<i32, i32> = HashMap::new();
        let mut category_id_map: HashMap<i32, i32> = HashMap::new();
        let mut tag_id_map: HashMap<i32, i32> = HashMap::new();

        for (orig_id, category) in data.categories {
            let model = categories::ActiveModel {
                name: Set(category.name),
                ordinal: Set(category.ordinal),
                board_id: Set(Some(new_board.id)),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert category")?;
            category_id_map.insert(*orig_id, inserted.id);
        }

        for (orig_tag_id, tag) in data.category_tags {
            let new_category_id = category_id_map.get(&tag.category_id).copied();
            let model = category_tags::ActiveModel {
                tag_name: Set(tag.name),
                color: Set(string_to_rgb_int(&tag.color)),
                ordinal: Set(tag.ordinal),
                category_id: Set(new_category_id),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert tag")?;
            tag_id_map.insert(*orig_tag_id, inserted.id);
        }

        for (orig_col_id, column) in data.columns {
            let model = columns::ActiveModel {
                name: Set(column.name),
                ordinal: Set(column.ordinal),
                board_id: Set(Some(new_board.id)),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert column")?;
            column_id_map.insert(*orig_col_id, inserted.id);
        }

        for (_, activity) in data.activities {
            let new_column_id = activity.column_id.and_then(|id| column_id_map.get(&id).copied());
            let model = activities::ActiveModel {
                name: Set(activity.name),
                body: Set(activity.body),
                column_id: Set(new_column_id),
                ordinal: Set(activity.ordinal),
                ..Default::default()
            };
            model.insert(&tr).await.context("failed to insert activity")?;
        }

        for (_, activity) in data.other_activities {
            let model = activities::ActiveModel {
                name: Set(activity.name),
                body: Set(activity.body),
                column_id: Set(None),
                ordinal: Set(activity.ordinal),
                ..Default::default()
            };
            model.insert(&tr).await.context("failed to insert other activity")?;
        }

        tr.commit().await.context("failed to commit transaction")?;

        Ok(new_board)
    }
}
