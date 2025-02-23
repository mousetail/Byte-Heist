-- Add migration script here
ALTER TABLE accounts
    ADD COLUMN admin BOOLEAN NOT NULL DEFAULT false;