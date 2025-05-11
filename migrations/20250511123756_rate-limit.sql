-- Add migration script here
ALTER TABLE accounts ADD last_creation_action TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now();