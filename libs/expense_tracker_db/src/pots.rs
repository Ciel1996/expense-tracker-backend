pub mod pots {
    use chrono::{DateTime, Utc};
    use crate::schema::pots;
    use crate::schema::pots_to_users;
    use diesel::{Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    /// Represents a pot. A pot is an accumulation of expenses and is owned by a single
    /// user. A pot can be shared with multiple users. The users can leave a pot
    /// anytime.
    /// The owner of a pot needs to invite other users to participate in a pot.
    #[derive(Serialize, Selectable, Queryable)]
    pub struct Pot {
        id: i32,
        owner_id: Uuid,
        name: String,
        default_currency_id: i32,
        archived: bool,
        created_at: DateTime<Utc>,
        archived_at: Option<DateTime<Utc>>,
    }

    impl Pot {
        pub fn new(
            id: i32,
            owner_id: Uuid,
            name: String,
            default_currency_id: i32,
            created_at: DateTime<Utc>
        ) -> Self {
            Self {
                id,
                owner_id,
                name,
                default_currency_id,
                archived: false,
                created_at,
                archived_at: None,
            }
        }

        /// Getter for id.
        pub fn id(&self) -> i32 {
            self.id
        }

        /// Getter for owner_id.
        pub fn owner_id(&self) -> Uuid {
            self.owner_id
        }

        /// Getter for name.
        pub fn name(&self) -> &str {
            &self.name
        }

        /// Getter for default_currency_id.
        pub fn default_currency_id(&self) -> i32 {
            self.default_currency_id
        }

        /// Getter for archived.
        pub fn is_archived(&self) -> bool { self.archived }

        /// Getter for created_at.
        pub fn created_at(&self) -> DateTime<Utc> { self.created_at }

        /// Getter for archived_at.
        pub fn archived_at(&self) -> Option<DateTime<Utc>> { self.archived_at }
    }

    /// This struct is used to create a new pot in the database.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = pots)]
    pub struct NewPot {
        owner_id: Uuid,
        name: String,
        default_currency_id: i32,
        created_at: DateTime<Utc>,
    }

    impl NewPot {
        /// Constructor
        pub fn new(owner_id: Uuid, name: String, default_currency_id: i32) -> Self {
            NewPot {
                owner_id,
                name,
                default_currency_id,
                created_at: Utc::now(),
            }
        }

        pub fn owner_id(&self) -> Uuid {
            self.owner_id
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn default_currency_id(&self) -> i32 {
            self.default_currency_id
        }
    }

    /// This struct is used to create a new pots_to_user relationship in the database.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = pots_to_users)]
    pub struct PotToUser {
        pot_id: i32,
        user_id: Uuid,
    }

    impl PotToUser {
        pub fn new(pot_id: i32, user_id: Uuid) -> Self {
            Self { pot_id, user_id }
        }

        pub fn pot_id(&self) -> i32 {
            self.pot_id
        }

        pub fn user_id(&self) -> Uuid {
            self.user_id
        }
    }
}
