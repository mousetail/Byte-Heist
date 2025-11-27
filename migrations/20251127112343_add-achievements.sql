CREATE TABLE achievements (
    user_id INTEGER NOT NULL REFERENCES accounts(id),
    achievement VARCHAR(64) NOT NULL,
    achieved BOOLEAN NOT NULL DEFAULT false,
    awarded_at TIMESTAMP WITH TIME ZONE,
    related_challenge INTEGER REFERENCES challenges(id),
    related_language VARCHAR(32),
    read BOOLEAN NOT NULL DEFAULT false
);

CREATE UNIQUE INDEX achievement_one_per_user ON achievements(user_id, achievement);