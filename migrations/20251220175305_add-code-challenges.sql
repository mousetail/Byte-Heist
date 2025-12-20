-- Add migration script here
ALTER TYPE challenge_category ADD VALUE 'code-challenge'; 

ALTER TABLE challenges ADD COLUMN unit varchar(12) NOT NULL DEFAULT 'Bytes';