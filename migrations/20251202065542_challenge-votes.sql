CREATE TABLE challenge_votes (
    id SERIAL NOT NULL PRIMARY KEY,
    author INTEGER NOT NULL REFERENCES accounts(id),
    challenge INTEGER NOT NULL REFERENCES challenges(id),
    is_upvote BOOLEAN NOT NULL
);

CREATE UNIQUE INDEX challenge_votes_unique ON challenge_votes(author, challenge);