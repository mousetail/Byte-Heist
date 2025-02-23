-- Add migration script here
ALTER TABLE solutions
    ADD COLUMN last_improved_date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now();

UPDATE solutions SET last_improved_date=updated_at;