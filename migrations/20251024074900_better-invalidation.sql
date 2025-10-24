-- Add migration script here
CREATE TABLE solution_retest_request (
    id SERIAL NOT NULL PRIMARY KEY,
    challenge INTEGER REFERENCES challenges(id),
    language VARCHAR(32),
    comment INTEGER NULL REFERENCES challenge_change_suggestions(comment) UNIQUE NULLS DISTINCT,
    author INTEGER NOT NULL REFERENCES accounts(id),
    solutions_passed INTEGER NOT NULL DEFAULT 0,
    solutions_failed INTEGER NOT NULL DEFAULT 0,
    solutions_timed_out INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    processed BOOLEAN NOT NULL DEFAULT false
);

ALTER TABLE solutions
    ADD COLUMN fail_reason INTEGER NULL REFERENCES solution_retest_request(id) DEFAULT NULL,
    ADD COLUMN time_out_count INTEGER NOT NULL DEFAULT 0;

ALTER TABLE solution_invalidation_log
    ADD COLUMN request INTEGER NULL REFERENCES solution_retest_request(id) DEFAULT NULL,
    ADD COLUMN timed_out BOOLEAN NOT NULL DEFAULT false;