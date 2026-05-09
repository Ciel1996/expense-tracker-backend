CREATE TABLE pot_templates
(
    id                  SERIAL PRIMARY KEY,
    owner_id            UUID REFERENCES users (id)         ON DELETE CASCADE NOT NULL,
    name                TEXT                               NOT NULL,
    default_currency_id INTEGER REFERENCES currencies (id) NOT NULL,
    create_at           TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE TABLE pot_template_users
(
  id                    SERIAL PRIMARY KEY,
  user_id               UUID REFERENCES users (id)          ON DELETE CASCADE NOT NULL,
  pot_template_id       INTEGER REFERENCES pot_templates    NOT NULL
);