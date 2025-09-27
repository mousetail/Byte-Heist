-- Add migration script here
CREATE MATERIALIZED VIEW user_scoring_info_per_language AS
WITH ranks AS (
    SELECT
        scores.author,
        scores.language,
        challenges.category,
        CAST(SUM(total_sols) AS bigint) as sols,
        CAST(COUNT(distinct challenges.id) as bigint) as distinct_challenges,
        CAST(SUM(score) AS bigint) as total_score,
        CAST(SUM(CASE
                WHEN rank = 1 THEN 1
                ELSE 0
            END
        ) AS bigint) as first_places,
        CAST(SUM(
            CASE
                WHEN rank <= total_sols / 10 + 1 THEN 1
                ELSE 0
            END
        ) AS bigint) as top_ten_percents
    FROM
        scores
    INNER JOIN
        challenges on scores.challenge = challenges.id
    WHERE challenges.status = 'public'
    GROUP BY scores.author, challenges.category, scores.language
    ORDER BY total_score DESC
)
SELECT 
    author,
    language,
    category,
    sols,
    total_score,
    first_places,
    top_ten_percents,
    distinct_challenges,
    rank() OVER (PARTITION BY category ORDER BY total_score DESC) as rank
FROM ranks
ORDER BY total_score DESC;
