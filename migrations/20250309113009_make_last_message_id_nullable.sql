-- Add migration script here
ALTER TABLE discord_messages
    ALTER COLUMN message_id DROP NOT NULL;