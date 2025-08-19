-- Add migration script here
ALTER TABLE user_activities ADD COLUMN 
    language varchar(32) NOT NULL;

DROP INDEX user_activites_one_per_date;

CREATE UNIQUE INDEX user_activites_one_per_date ON user_activities (account, challenge, date, language);