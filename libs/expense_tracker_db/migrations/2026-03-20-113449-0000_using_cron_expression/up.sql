ALTER TABLE pot_templates
DROP COLUMN create_at;

ALTER TABLE pot_templates
DROP COLUMN Occurrence;

ALTER TABLE pot_templates
ADD COLUMN cron_expression TEXT NOT NULL;