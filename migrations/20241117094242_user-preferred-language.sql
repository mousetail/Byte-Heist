-- Add migration script here
ALTER TABLE accounts
    ADD COLUMN preferred_language VARCHAR(32) NOT NULL DEFAULT 'python';
