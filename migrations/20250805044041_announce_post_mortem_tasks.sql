-- Add migration script here
ALTER TABLE challenges
    ADD COLUMN post_mortem_announced BOOLEAN DEFAULT false,
    ADD COLUMN post_mortem_warning_announced BOOLEAN DEFAULT false
;