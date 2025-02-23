-- Add migration script here
ALTER TABLE solutions
ADD COLUMN validated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
ADD COLUMN valid BOOLEAN NOT NULL DEFAULT true;

CREATE TABLE solution_invalidation_log (
    id SERIAL NOT NULL PRIMARY KEY,
    solution INTEGER NOT NULL REFERENCES solutions(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    pass BOOLEAN NOT NULL
);
