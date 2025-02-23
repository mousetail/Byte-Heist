-- Add migration script here
CREATE TABLE discord_messages (
    id SERIAL NOT NULL PRIMARY KEY,
    language VARCHAR(64) NOT NULL,
    challenge INTEGER NOT NULL REFERENCES challenges (id),
    author INTEGER NOT NULL REFERENCES accounts (id),
    previous_author INTEGER REFERENCES accounts (id),
    previous_author_score INTEGER,
    score INTEGER NOT NULL,
    message_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL
)