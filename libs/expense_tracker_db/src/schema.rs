// @generated automatically by Diesel CLI.

diesel::table! {
    currencies (id) {
        id -> Int4,
        name -> Text,
        symbol -> Text,
    }
}

diesel::table! {
    expense_splits (expense_id, user_id) {
        expense_id -> Int4,
        user_id -> Uuid,
        amount -> Float8,
        is_paid -> Bool,
    }
}

diesel::table! {
    expenses (id) {
        id -> Int4,
        owner_id -> Uuid,
        pot_id -> Int4,
        description -> Text,
        currency_id -> Int4,
    }
}

diesel::table! {
    pot_template_users (id) {
        id -> Int4,
        user_id -> Uuid,
        pot_template_id -> Int4,
    }
}

diesel::table! {
    pot_templates (id) {
        id -> Int4,
        owner_id -> Uuid,
        name -> Text,
        default_currency_id -> Int4,
        create_at -> Timestamptz,
    }
}

diesel::table! {
    pots (id) {
        id -> Int4,
        owner_id -> Uuid,
        name -> Text,
        default_currency_id -> Int4,
        created_at -> Timestamptz,
        archived_at -> Nullable<Timestamptz>,
        archived -> Bool,
    }
}

diesel::table! {
    pots_to_users (pot_id, user_id) {
        pot_id -> Int4,
        user_id -> Uuid,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Text,
    }
}

diesel::joinable!(expense_splits -> expenses (expense_id));
diesel::joinable!(expense_splits -> users (user_id));
diesel::joinable!(expenses -> currencies (currency_id));
diesel::joinable!(expenses -> pots (pot_id));
diesel::joinable!(expenses -> users (owner_id));
diesel::joinable!(pot_template_users -> pot_templates (pot_template_id));
diesel::joinable!(pot_template_users -> users (user_id));
diesel::joinable!(pot_templates -> currencies (default_currency_id));
diesel::joinable!(pot_templates -> users (owner_id));
diesel::joinable!(pots -> currencies (default_currency_id));
diesel::joinable!(pots -> users (owner_id));
diesel::joinable!(pots_to_users -> pots (pot_id));
diesel::joinable!(pots_to_users -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    currencies,
    expense_splits,
    expenses,
    pot_template_users,
    pot_templates,
    pots,
    pots_to_users,
    users,
);
