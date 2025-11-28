CREATE UNIQUE INDEX scores_pk ON scores(id);
CREATE UNIQUE INDEX user_scoring_info_pk ON user_scoring_info(author, category);
CREATE UNIQUE INDEX user_scoring_info_per_language_pk ON user_scoring_info_per_language(author, language, category);

ALTER TABLE achievements ADD COLUMN progress BIGINT, ADD COLUMN total BIGINT;

ALTER TABLE account_oauth_codes ALTER COLUMN refresh_token TYPE VARCHAR(128);