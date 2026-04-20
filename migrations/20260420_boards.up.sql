-- Create boards table
CREATE TABLE boards (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Add board_id to columns table
ALTER TABLE columns ADD COLUMN board_id INTEGER;

-- Add board_id to categories table
ALTER TABLE categories ADD COLUMN board_id INTEGER;

-- Create default board
INSERT INTO boards (name, created_at, updated_at)
VALUES ('默认看板', datetime('now'), datetime('now'));

-- Update existing columns and categories to use default board
UPDATE columns SET board_id = 1 WHERE board_id IS NULL;
UPDATE categories SET board_id = 1 WHERE board_id IS NULL;

-- Add foreign key constraints (SQLite doesn't support adding FK with ALTER TABLE, so we need to recreate tables)
-- But for simplicity, we'll just add the board_id column without enforcing FK at database level
