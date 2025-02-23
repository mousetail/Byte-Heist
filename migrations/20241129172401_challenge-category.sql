-- Add migration script here
CREATE TYPE challenge_category AS ENUM ('code-golf', 'restricted-source', 'private');
create TYPE challenge_status AS ENUM ('draft', 'beta', 'public', 'private');

ALTER TABLE challenges
    ADD COLUMN status challenge_status DEFAULT 'beta',
    ADD COLUMN category challenge_category DEFAULT 'restricted-source',
    ADD COLUMN tags jsonb DEFAULT '[]' CHECK (tags IS JSON ARRAY);