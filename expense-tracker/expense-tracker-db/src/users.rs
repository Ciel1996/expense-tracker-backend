pub mod users {
    use diesel::{Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};
    use crate::schema::users;

    /// Defines a user of the systems.
    /// A user can hold pots, expenses and so on.
    #[derive(Serialize, Selectable, Queryable)]
    pub struct User {
        id: i32,
        name: String,
    }

    impl User {
        /// Constructor for User
        pub fn new(id: i32, name: String) -> User {
            User { id, name }
        }

        /// Getter for user_id
        pub fn id(&self) -> i32 {
            self.id
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }

    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = users)]
    pub struct NewUser {
        name: String,
    }

    impl NewUser {
        pub fn new(name: String) -> NewUser {
            NewUser { name }
        }
    }
}