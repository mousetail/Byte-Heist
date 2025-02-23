-- Add migration script here
ALTER TABLE challenges
    ADD COLUMN post_mortem_date TIMESTAMP WITH TIME ZONE DEFAULT null;

ALTER TABLE solutions
    ADD COLUMN runtime REAL NOT NULL DEFAULT 3.0;