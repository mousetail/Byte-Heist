-- Add migration script here
ALTER TABLE challenge_comments DROP COLUMN diff;

CREATE TYPE challenge_diff_field AS ENUM ('description', 'judge', 'example-code');
CREATE TYPE challenge_diff_status AS ENUM ('active', 'accepted', 'rejected');

CREATE TABLE challenge_change_suggestions(
    id SERIAL NOT NULL PRIMARY KEY,
    comment INTEGER NOT NULL REFERENCES challenge_comments(id) UNIQUE,
    challenge INTEGER NOT NULL REFERENCES challenges(id),
    field challenge_diff_field NOT NULL,
    status challenge_diff_status NOT NULL,
    new_value TEXT NOT NULL,
    old_value TEXT NOT NULL
);

CREATE UNIQUE INDEX challenge_current_suggestion ON challenge_change_suggestions(challenge, field) where status='active';

ALTER TABLE challenge_comments ADD COLUMN
    last_vote_time TIMESTAMP WITH TIME ZONE NOT NULL default now(),
    last_vote_processed_time TIMESTAMP WITH TIME ZONE NOT NOT NULL default now();