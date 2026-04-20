use std::collections::HashMap;

use crate::{
    commands::board::{
        ColumnActivityExport, ColumnExport, CreateBoardInput, CategoryExport, CategoryTagExport,
        ExportBoardOutput, ActivityExport, ImportBoardInput, OtherTagExport, UpdateBoardInput,
    },
    errors::AppError,
    utils::coloring::rgb_string_to_int,
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
    pub async fn all_boards(db: &DbConn) -> Result<Vec<BoardModel>, AppError> {
        let res = Board::find()
            .all(db)
            .await
            .context("failed to fetch all boards")?;
        Ok(res)
    }

    pub async fn get_board_by_id(db: &DbConn, id: i32) -> Result<BoardModel, AppError> {
        let res = Board::find_by_id(id)
            .one(db)
            .await
            .context("failed to fetch board by id")?
            .ok_or(AppError::RowNotFound)?;
        Ok(res)
    }

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

        let mut columns: HashMap<i32, ColumnExport> = HashMap::new();
        let mut activities_map: HashMap<i32, ColumnActivityExport> = HashMap::new();
        let mut other_activities_map: HashMap<i32, ActivityExport> = HashMap::new();

        for (column, col_activities) in columns_data {
            let activity_ids: Vec<i32> = col_activities.iter().map(|a| a.id).collect();
            columns.insert(
                column.id,
                ColumnExport {
                    name: column.name,
                    ordinal: column.ordinal,
                    activities: activity_ids,
                },
            );

            for activity in col_activities {
                activities_map.insert(
                    activity.id,
                    ColumnActivityExport {
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
                ActivityExport {
                    name: activity.name,
                    body: activity.body,
                    ordinal: activity.ordinal,
                    tags: vec![],
                },
            );
        }

        let mut categories: HashMap<i32, CategoryExport> = HashMap::new();
        let mut category_tags_map: HashMap<i32, CategoryTagExport> = HashMap::new();

        for (category, tags) in categories_data {
            let tag_ids: Vec<i32> = tags.iter().map(|t| t.id).collect();
            categories.insert(
                category.id,
                CategoryExport {
                    name: category.name,
                    ordinal: category.ordinal,
                    tags: tag_ids,
                },
            );

            for tag in tags {
                category_tags_map.insert(
                    tag.id,
                    CategoryTagExport {
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
            activities: activities_map,
            other_activities: other_activities_map,
            categories,
            category_tags: category_tags_map,
            other_tags: HashMap::new(),
        })
    }
}

pub struct Mutation;

impl Mutation {
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

    pub async fn delete_board(db: &DbConn, board_id: i32) -> Result<(), AppError> {
        let tr = db.begin().await.context("failed to begin transaction")?;

        let column_ids = columns::Entity::find()
            .filter(columns::Column::BoardId.eq(board_id))
            .all(&tr)
            .await
            .context("failed to get column ids")?
            .iter()
            .map(|c| c.id)
            .collect::<Vec<i32>>();

        let category_ids = categories::Entity::find()
            .filter(categories::Column::BoardId.eq(board_id))
            .all(&tr)
            .await
            .context("failed to get category ids")?
            .iter()
            .map(|c| c.id)
            .collect::<Vec<i32>>();

        if !column_ids.is_empty() {
            activities::Entity::delete_many()
                .filter(activities::Column::ColumnId.is_in(column_ids))
                .exec(&tr)
                .await
                .context("failed to delete activities")?;
        }

        if !category_ids.is_empty() {
            category_tags::Entity::delete_many()
                .filter(category_tags::Column::CategoryId.is_in(category_ids))
                .exec(&tr)
                .await
                .context("failed to delete category tags")?;
        }

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

        for (cat_id, category) in &exported.categories {
            let model = categories::ActiveModel {
                name: Set(category.name.clone()),
                ordinal: Set(category.ordinal),
                board_id: Set(Some(new_board.id)),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert category")?;
            category_id_map.insert(*cat_id, inserted.id);
        }

        for (tag_id, tag) in &exported.category_tags {
            let new_category_id = category_id_map.get(&tag.category_id).copied();
            let color_value = match rgb_string_to_int(&tag.color) {
                Ok(c) => c,
                Err(_) => 0,
            };
            let model = category_tags::ActiveModel {
                tag_name: Set(tag.name.clone()),
                color: Set(color_value),
                ordinal: Set(tag.ordinal),
                category_id: Set(new_category_id),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert tag")?;
            tag_id_map.insert(*tag_id, inserted.id);
        }

        for (col_id, column) in &exported.columns {
            let model = columns::ActiveModel {
                name: Set(column.name.clone()),
                ordinal: Set(column.ordinal),
                board_id: Set(Some(new_board.id)),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert column")?;
            column_id_map.insert(*col_id, inserted.id);
        }

        for (_, activity) in &exported.activities {
            let new_column_id = column_id_map.get(&activity.column_id).copied();
            let model = activities::ActiveModel {
                name: Set(activity.name.clone()),
                body: Set(activity.body.clone()),
                column_id: Set(new_column_id),
                ordinal: Set(activity.ordinal),
                ..Default::default()
            };
            model.insert(&tr).await.context("failed to insert activity")?;
        }

        for (_, activity) in &exported.other_activities {
            let model = activities::ActiveModel {
                name: Set(activity.name.clone()),
                body: Set(activity.body.clone()),
                column_id: Set(None),
                ordinal: Set(activity.ordinal),
                ..Default::default()
            };
            model.insert(&tr).await.context("failed to insert other activity")?;
        }

        tr.commit().await.context("failed to commit transaction")?;

        Ok(new_board)
    }

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

        for (orig_id, category) in &data.categories {
            let model = categories::ActiveModel {
                name: Set(category.name.clone()),
                ordinal: Set(category.ordinal),
                board_id: Set(Some(new_board.id)),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert category")?;
            category_id_map.insert(*orig_id, inserted.id);
        }

        for (orig_tag_id, tag) in &data.category_tags {
            let new_category_id = category_id_map.get(&tag.category_id).copied();
            let color_value = match rgb_string_to_int(&tag.color) {
                Ok(c) => c,
                Err(_) => 0,
            };
            let model = category_tags::ActiveModel {
                tag_name: Set(tag.name.clone()),
                color: Set(color_value),
                ordinal: Set(tag.ordinal),
                category_id: Set(new_category_id),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert tag")?;
            tag_id_map.insert(*orig_tag_id, inserted.id);
        }

        for (orig_col_id, column) in &data.columns {
            let model = columns::ActiveModel {
                name: Set(column.name.clone()),
                ordinal: Set(column.ordinal),
                board_id: Set(Some(new_board.id)),
                ..Default::default()
            };
            let inserted = model.insert(&tr).await.context("failed to insert column")?;
            column_id_map.insert(*orig_col_id, inserted.id);
        }

        for (_, activity) in &data.activities {
            let new_column_id = column_id_map.get(&activity.column_id).copied();
            let model = activities::ActiveModel {
                name: Set(activity.name.clone()),
                body: Set(activity.body.clone()),
                column_id: Set(new_column_id),
                ordinal: Set(activity.ordinal),
                ..Default::default()
            };
            model.insert(&tr).await.context("failed to insert activity")?;
        }

        for (_, activity) in &data.other_activities {
            let model = activities::ActiveModel {
                name: Set(activity.name.clone()),
                body: Set(activity.body.clone()),
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
