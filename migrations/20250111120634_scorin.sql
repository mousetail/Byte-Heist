-- Add migration script here

CREATE MATERIALIZED VIEW scores AS
WITH ranks AS (SELECT
    author,
    challenge,
    language,
    (SELECT COUNT(*) FROM solutions as s2 WHERE s2.language = solutions.language AND s2.challenge = solutions.challenge) as total_sols,
    rank() OVER (PARTITION BY language, challenge ORDER BY score ASC) as rank
FROM solutions)
SELECT author, challenge, language, total_sols, rank, (total_sols - rank + 1) / total_sols * 1000 as score
FROM ranks;