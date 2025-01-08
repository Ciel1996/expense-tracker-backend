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
        description -> Text,
        amount -> Float8,
        currency_id -> Int4,
        is_paid -> Bool,
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
    users (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::joinable!(expenses -> currencies (currency_id));
diesel::joinable!(expenses -> users (owner_id));
diesel::joinable!(pots -> currencies (default_currency_id));
diesel::joinable!(pots -> users (owner_id));

diesel::allow_tables_to_appear_in_same_query!(
    currencies,
    expenses,
    pots,
    users,
);
