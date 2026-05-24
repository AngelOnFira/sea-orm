-- Schema for the typed-PK chat example.
--
-- Discord-shaped data model that exercises the cases per-entity PK
-- newtypes are designed to catch:
--   * cross-entity ID confusion (every snowflake i64 is a distinct type)
--   * multi-FK columns to the same parent (junction tables use role wrappers)
--   * self-references (message replies)
--   * composite primary keys (membership, reactions)
--
-- Entities under src/entity/ are generated from this file via:
--
--   sqlite3 /tmp/typed_pk_chat.db < examples/basic_typed_pk/chat.sql
--   sea-orm-cli generate entity \
--       --database-url sqlite:///tmp/typed_pk_chat.db \
--       --with-pk-newtypes \
--       --output-dir examples/basic_typed_pk/src/entity

CREATE TABLE guild (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE user (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE channel (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    guild_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    FOREIGN KEY (guild_id) REFERENCES guild (id)
);

-- Composite PK whose components are both typed FKs into other tables.
CREATE TABLE member (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    nickname VARCHAR(255),
    PRIMARY KEY (guild_id, user_id),
    FOREIGN KEY (guild_id) REFERENCES guild (id),
    FOREIGN KEY (user_id) REFERENCES user (id)
);

-- Self-ref via reply_to_message_id; two FKs to user (author + mention).
-- Neither of those user FKs is a PK column, so codegen does NOT emit role
-- wrappers for them. They share the parent's UserId type.
CREATE TABLE message (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    channel_id BIGINT NOT NULL,
    author_id BIGINT NOT NULL,
    mention_user_id BIGINT,
    reply_to_message_id BIGINT,
    content TEXT NOT NULL,
    FOREIGN KEY (channel_id) REFERENCES channel (id),
    FOREIGN KEY (author_id) REFERENCES user (id),
    FOREIGN KEY (mention_user_id) REFERENCES user (id),
    FOREIGN KEY (reply_to_message_id) REFERENCES message (id)
);

-- Three-column composite PK with two typed FKs to other entities and a
-- raw string emoji.
CREATE TABLE reaction (
    message_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    emoji VARCHAR(255) NOT NULL,
    PRIMARY KEY (message_id, user_id, emoji),
    FOREIGN KEY (message_id) REFERENCES message (id),
    FOREIGN KEY (user_id) REFERENCES user (id)
);

-- Junction with two PK columns both FK-referencing user.id. This is the
-- canonical role-wrapper case: codegen emits per-column wrapper structs
-- (`UserFollowerPkUserId`, `UserFollowerPkFollowerId`) so positional
-- swaps fail to compile.
CREATE TABLE user_follower (
    user_id BIGINT NOT NULL,
    follower_id BIGINT NOT NULL,
    PRIMARY KEY (user_id, follower_id),
    FOREIGN KEY (user_id) REFERENCES user (id),
    FOREIGN KEY (follower_id) REFERENCES user (id)
);
