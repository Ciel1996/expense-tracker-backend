UPDATE pots SET name = LEFT(name, 24) WHERE length(name) > 24;
UPDATE expenses SET description = LEFT(description, 24) WHERE length(description) > 24;

ALTER TABLE pots
ADD CONSTRAINT name_length_check CHECK (length(name) <= 24);

ALTER TABLE expenses
ADD CONSTRAINT description_length_check CHECK (length(description) <= 24);