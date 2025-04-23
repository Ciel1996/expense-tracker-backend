pub mod pots {
    use diesel::{Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use crate::schema::pots;

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
    }

    impl Pot {
        pub fn new(
            id: i32,
            owner_id: Uuid,
            name: String,
            default_currency_id: i32
        ) -> Self {
            Self {
                id,
                owner_id,
                name,
                default_currency_id
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

    }

    /// This struct is used to create a new pot in the database.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = pots)]
    pub struct NewPot {
        owner_id: Uuid,
        name: String,
        default_currency_id: i32,
    }

    impl NewPot {
        /// Constructor
        pub fn new(
            owner_id: Uuid,
            name: String,
            default_currency_id: i32,
        ) -> Self {
            NewPot {
                owner_id,
                name,
                default_currency_id
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
}