-- Add migration script here

-- create user table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    -- hashed argon2 password, length 97
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM (
    'single',
    'group',
    'private_channel',
    'public_channel'
);

-- create chat table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64),
    type chat_type NOT NULL,

    -- user id list
    members BIGINT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    images TEXT[],
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index for message for chat_id and created_at order by created_at DESC
CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);

-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id);
