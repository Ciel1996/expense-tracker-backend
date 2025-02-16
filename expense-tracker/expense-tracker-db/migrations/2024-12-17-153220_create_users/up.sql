-- Your SQL goes here
CREATE TABLE users
(
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE currencies
(
    id     SERIAL PRIMARY KEY,
    name   TEXT NOT NULL,
    symbol TEXT NOT NULL
);

CREATE TABLE pots
(
    id                  SERIAL PRIMARY KEY,
    owner_id            INTEGER REFERENCES users (id)      NOT NULL,
    name                TEXT                               NOT NULL,
    default_currency_id INTEGER REFERENCES currencies (id) NOT NULL
);

CREATE TABLE expenses
(
    id          SERIAL PRIMARY KEY,
    owner_id    INTEGER REFERENCES users (id)      NOT NULL,
    pot_id      INTEGER REFERENCES pots (id)       NOT NULL,
    description TEXT                               NOT NULL,
    currency_id INTEGER REFERENCES currencies (id) NOT NULL
);

CREATE TABLE expense_splits
(
    expense_id INTEGER REFERENCES expenses (id) ON DELETE CASCADE NOT NULL ,
    user_id    INTEGER REFERENCES users (id)    ON DELETE CASCADE NOT NULL,
    amount     DOUBLE PRECISION                 NOT NULL,
    is_paid    BOOLEAN                          NOT NULL,
    PRIMARY KEY (expense_id, user_id)
);

CREATE TABLE pots_to_users
(
    pot_id  INTEGER REFERENCES pots (id)  NOT NULL,
    user_id INTEGER REFERENCES users (id) NOT NULL,
    PRIMARY KEY (pot_id, user_id)
);

-- initial seeding
INSERT INTO currencies (name, symbol)
VALUES ('SwissFranc', 'CHF'),
       ('Euro', '€'),
       ('UsDollar', 'US$');