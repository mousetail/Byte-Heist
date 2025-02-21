-- Add migration script here
ALTER TABLE solutions
    ADD COLUMN is_post_mortem boolean DEFAULT false;

DROP MATERIALIZED VIEW scores;

CREATE MATERIALIZED VIEW scores AS
WITH ranks AS (
    SELECT
        author,
        challenge,
        language,
        (SELECT COUNT(*) FROM solutions as s2 WHERE s2.language = solutions.language AND s2.challenge = solutions.challenge AND s2.valid) as total_sols,
        rank() OVER (PARTITION BY language, challenge, valid ORDER BY score ASC) as rank
    FROM solutions
    WHERE solutions.valid AND NOT solutions.is_post_mortem
)
SELECT author, challenge, language, total_sols, rank, (total_sols - rank + 1) * 1000 / total_sols as score
FROM ranks;