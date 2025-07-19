ALTER TABLE pots_to_users
DROP CONSTRAINT pots_to_users_pkey;

ALTER TABLE pots_to_users
DROP CONSTRAINT pots_to_users_pot_id_fkey,
ADD CONSTRAINT pots_to_users_pot_id_fkey
    FOREIGN KEY (pot_id)
    REFERENCES pots(id)
    ON DELETE CASCADE;

ALTER TABLE pots_to_users
DROP CONSTRAINT pots_to_users_user_id_fkey,
ADD CONSTRAINT pots_to_users_user_id_fkey
    FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE CASCADE;

ALTER TABLE pots_to_users
    ADD CONSTRAINT pots_to_users_pkey
        PRIMARY KEY (pot_id, user_id);