-- Add migration script here
CREATE TABLE user_activities (
    id BIGSERIAL NOT NULL PRIMARY KEY,
    account INTEGER NOT NULL REFERENCES accounts(id),
    challenge INTEGER NOT NULL REFERENCES challenges(id),
    old_score INTEGER NULL,
    new_score INTEGER NOT NULL,
    date DATE default(now()),

    created_at TIMESTAMP WITH TIME ZONE NOT NULL default(now()),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL default(now())
);

CREATE TRIGGER update_user_activity_changetimestamp BEFORE UPDATE
    ON user_activities FOR EACH ROW EXECUTE PROCEDURE 
    update_modified_column();

CREATE UNIQUE INDEX user_activites_one_per_date ON user_activities (account, challenge, date);