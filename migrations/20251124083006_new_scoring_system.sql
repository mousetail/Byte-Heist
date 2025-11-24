-- Add migration script here
DROP MATERIALIZED view if Exists user_scoring_info RESTRICT;
DROP MATERIALIZED VIEW if Exists user_scoring_info_per_language RESTRICT;
DROP MATERIALIZED VIEW if exists scores RESTRICT;

CREATE MATERIALIZED VIEW scores AS
WITH ranks AS (
    SELECT
        author,
        challenge,
        language,
        score,
        valid,
        cast((SELECT COUNT(*) FROM solutions as s2 WHERE s2.language = solutions.language AND s2.challenge = solutions.challenge and s2.valid)as integer) as total_sols,
        rank() OVER peers as rank
    FROM solutions
    WHERE solutions.valid and not solutions.is_post_mortem
    window peers as (PARTITION BY language, challenge, valid ORDER BY score asc rows between unbounded preceding and unbounded following)
),
percentiles AS (
    SELECT
        author,
        challenge,
        language,
        score,
        total_sols,
        rank,
        (case WHEN total_sols > 2
        	then nth_value(score, total_sols / 2 + 1)
            	OVER peers
            else 9999
         end) as percentile_50th,
        (case WHEN total_sols > 1
	        then nth_value(score, total_sols * 9 / 10 + 1)
	            OVER peers
	        else 9999
	     end) as percentile_90th,
        (case WHEN total_sols > 9 then 
        	nth_value(score, total_sols / 10 + 1)
            	OVER peers
            else 9999
        end) as percentile_10th
    FROM ranks
    window peers as (PARTITION BY language, challenge ORDER BY score asc rows between unbounded preceding and unbounded following)
)
SELECT
    author,
    challenge,
    language,
    total_sols,
    rank,
    -- score consists of 4 parts
    -- first, 10 points for all first place sols
    (case when "rank" = 1 then 10 else 0 end) +
    -- Next, 1/4 point for each byte over the bottom 90th percentile
    least(greatest((coalesce(percentile_90th, 9999) - score), 0)/4, 50) +
    -- Next, 1/2 point for each byte over the 50th percentile
    least(greatest((coalesce(percentile_50th, 9999) - score), 0)/2, 50) +
    -- Next, a byte for each point over the top 90th percentile
    least(greatest((coalesce(percentile_10th, 9999) - score), 0), 49) +
    1
     as score
FROM percentiles;

CREATE MATERIALIZED VIEW user_scoring_info AS
WITH ranks AS (
    SELECT
        scores.author,
        challenges.category,
        CAST(COUNT(*) AS bigint) as sols,
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
    GROUP BY scores.author, challenges.category
    ORDER BY total_score DESC
)
SELECT 
    author,
    category,
    sols,
    total_score,
    first_places,
    top_ten_percents,
    distinct_challenges,
    rank() OVER (PARTITION BY category ORDER BY total_score DESC) as rank
FROM ranks
ORDER BY total_score DESC;

CREATE MATERIALIZED VIEW user_scoring_info_per_language AS
WITH ranks AS (
    SELECT
        scores.author,
        scores.language,
        challenges.category,
        CAST(COUNT(*) AS bigint) as sols,
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
