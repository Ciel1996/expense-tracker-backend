pub mod users {
    use crate::schema::users;
    use diesel::{Insertable, Queryable, Selectable};
    use serde::Serialize;
    use uuid::Uuid;

    /// Defines a user of the systems.
    /// A user can hold pots, expenses and so on.
    #[derive(Serialize, Selectable, Queryable, Insertable)]
    pub struct User {
        id: Uuid,
        name: String,
    }

    impl User {
        /// Constructor for User
        pub fn new(uuid: Uuid, name: String) -> User {
            User { id: uuid, name }
        }

        /// Getter for user_id
        pub fn id(&self) -> Uuid {
            self.id
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }
}
