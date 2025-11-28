ALTER TABLE challenges ADD COLUMN go_live_date TIMESTAMP WITH TIME ZONE;

CREATE MATERIALIZED VIEW achievement_stats AS
    SELECT
        achievement,
        count(*) as total_awarded
    FROM achievements
    WHERE achieved
    GROUP BY achievement;

CREATE UNIQUE INDEX achievement_stats_pk ON achievement_stats(achievement);