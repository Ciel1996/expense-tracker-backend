// @generated automatically by Diesel CLI.

diesel::table! {
    currencies (id) {
        id -> Int4,
        name -> Text,
        symbol -> Text,
    }
}

diesel::table! {
    expenses (id) {
        id -> Int4,
        owner_id -> Int4,
        pot_id -> Int4,
        description -> Text,
        currency_id -> Int4,
    }
}

diesel::table! {
    pots (id) {
        id -> Int4,
        owner_id -> Int4,
        name -> Text,
        default_currency_id -> Int4,
    }
}

diesel::table! {
    pots_to_users (pot_id, user_id) {
        pot_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    splits (expense_id, user_id) {
        expense_id -> Int4,
        user_id -> Int4,
        amount -> Float8,
        is_paid -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::joinable!(expenses -> currencies (currency_id));
diesel::joinable!(expenses -> pots (pot_id));
diesel::joinable!(expenses -> users (owner_id));
diesel::joinable!(pots -> currencies (default_currency_id));
diesel::joinable!(pots -> users (owner_id));
diesel::joinable!(pots_to_users -> pots (pot_id));
diesel::joinable!(pots_to_users -> users (user_id));
diesel::joinable!(splits -> expenses (expense_id));
diesel::joinable!(splits -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    currencies,
    expenses,
    pots,
    pots_to_users,
    splits,
    users,
);
