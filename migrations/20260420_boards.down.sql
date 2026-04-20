-- SQLite doesn't support dropping columns easily
-- So we'll just drop and recreate the tables without board_id

-- Note: This will lose data! Only use this for development.
DROP TABLE IF EXISTS activity_tags;
DROP TABLE IF EXISTS category_tags;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS activities;
DROP TABLE IF EXISTS columns;
DROP TABLE IF EXISTS boards;

-- Recreate original tables
CREATE TABLE columns (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    ordinal INT NOT NULL
);

CREATE TABLE activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    body TEXT,
    column_id INTEGER,
    ordinal INT NOT NULL,
    FOREIGN KEY (column_id) REFERENCES columns(id) ON DELETE SET NULL
);

CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    ordinal INT NOT NULL
);

CREATE TABLE category_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    tag_name TEXT NOT NULL,
    category_id INTEGER,
    color INTEGER NOT NULL,
    ordinal INT NOT NULL,
    UNIQUE (tag_name, category_id),
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE TABLE activity_tags (
    activity_id INTEGER NOT NULL,
    category_tag_id INTEGER NOT NULL,
    PRIMARY KEY (activity_id, category_tag_id),
    FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE CASCADE,
    FOREIGN KEY (category_tag_id) REFERENCES category_tags(id) ON DELETE CASCADE
);
