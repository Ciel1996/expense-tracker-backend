ALTER TABLE pot_templates
DROP CONSTRAINT name_length_check;

ALTER TABLE pot_template_users
DROP CONSTRAINT pot_template_users_pot_template_id_fkey,
    ADD CONSTRAINT pot_template_users_pot_template_id_fkey
     FOREIGN KEY (pot_template_id)
     REFERENCES pot_templates (id);
