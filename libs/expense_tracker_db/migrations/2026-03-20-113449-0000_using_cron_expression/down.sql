ALTER TABLE pot_templates
DROP COLUMN cron_expression;

ALTER TABLE pot_templates
    ADD COLUMN create_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL

ALTER TABLE pot_templates
    ADD COLUMN Occurrence VARCHAR NOT NULL DEFAULT 'once';