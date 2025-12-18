-- Add migration script here
ALTER TABLE accounts ADD COLUMN referrer VARCHAR(128) DEFAULT NULL;

CREATE TYPE challenge_difficulty AS ENUM ('easy', 'medium', 'hard');

ALTER TABLE challenges ADD COLUMN difficulty challenge_difficulty  NOT NULL DEFAULT 'medium';