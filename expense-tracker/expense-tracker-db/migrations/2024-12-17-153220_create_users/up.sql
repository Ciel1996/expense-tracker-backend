-- Your SQL goes here
CREATE TABLE users
(
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE currencies
(
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL
);

CREATE TABLE expenses (
    id SERIAL PRIMARY KEY,
    owner_id INTEGER REFERENCES users(id) NOT NULL,
    description TEXT NOT NULL,
    amount DOUBLE PRECISION NOT NULL,
    currency_id INTEGER REFERENCES currencies(id) NOT NULL,
    is_paid BOOLEAN NOT NULL
);

-- TODO: we need to connect a pot to a list of expenses
CREATE TABLE pots (
    id SERIAL PRIMARY KEY,
    owner_id INTEGER REFERENCES users(id) NOT NULL,
    name TEXT NOT NULL,
    default_currency_id INTEGER REFERENCES currencies(id) NOT NULL
);

-- initial seeding
INSERT INTO currencies (name, symbol) VALUES
    ('SwissFranc', 'CHF'),
    ('Euro', '€'),
    ('UsDollar', 'US$');