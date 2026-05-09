UPDATE pot_templates SET name = LEFT(name, 24) WHERE length(name) > 24;

ALTER TABLE pot_templates
ADD CONSTRAINT name_length_check CHECK (length(name) <= 24);

ALTER TABLE pot_template_users
DROP CONSTRAINT pot_template_users_pot_template_id_fkey,
ADD CONSTRAINT pot_template_users_pot_template_id_fkey
     FOREIGN KEY (pot_template_id)
     REFERENCES pot_templates(id) ON DELETE CASCADE;