-- Add migration script here
CREATE TABLE challenge_comments (
    id SERIAL NOT NULL PRIMARY KEY,
    challenge INTEGER NOT NULL REFERENCES challenges(id),
    parent INTEGER NULL REFERENCES challenge_comments(id),
    author INTEGER NOT NULL REFERENCES accounts(id),
    message TEXT NOT NULL,
    diff TEXT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE TABLE challenge_comment_votes (
    id SERIAL NOT NULL PRIMARY KEY,
    author INTEGER NOT NULL REFERENCES accounts(id),
    comment INTEGER NOT NULL REFERENCES challenge_comments(id),
    is_upvote BOOLEAN NOT NULL
);

CREATE UNIQUE INDEX challenge_comment_votes_unique ON challenge_comment_votes(author, comment);