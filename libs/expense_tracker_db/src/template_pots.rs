pub mod template_pots {
    use crate::schema::pot_templates;
    use crate::schema::pot_template_users;
    use chrono::{DateTime, Utc};
    use diesel::{Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Clone, Serialize, Selectable, Queryable)]
    #[diesel(table_name = pot_templates)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct PotTemplate {
        id: i32,
        owner_id: Uuid,
        name: String,
        default_currency_id: i32,
        create_at: DateTime<Utc>
    }
    
    impl PotTemplate {
        pub fn new(
            id: i32,
            owner_id: Uuid,
            name: String,
            default_currency_id: i32,
            create_at: DateTime<Utc>
        ) -> Self {
            Self {
                id,
                owner_id,
                name,
                default_currency_id,
                create_at,
            }
        }
        
        pub fn id(&self) -> i32 {
            self.id
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

        pub fn create_at(&self) -> DateTime<Utc> {
            self.create_at
        }
    }

    #[derive(Clone, Deserialize, Insertable)]
    #[diesel(table_name = pot_templates)]
    pub struct NewPotTemplate {
        owner_id: Uuid,
        name: String,
        default_currency_id: i32,
        create_at: DateTime<Utc>,
    }

    impl NewPotTemplate {
        pub fn new(
            owner_id: Uuid, 
            name: String,
            default_currency_id: i32,
            create_at: DateTime<Utc>
        ) -> Self {
            Self {
                owner_id,
                name,
                default_currency_id,
                create_at
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

        pub fn create_at(&self) -> DateTime<Utc> {
            self.create_at
        }
    }

    #[derive(Serialize, Selectable, Queryable)]
    #[diesel(table_name = pot_template_users)]
    pub struct PotTemplateUser {
        id: i32,
        user_id: Uuid,
        pot_template_id: i32
    }

    impl PotTemplateUser {
        pub fn new(id: i32, user_id: Uuid, pot_template_id: i32) -> Self {
            Self { id, user_id, pot_template_id }
        }

        pub fn id(&self) -> i32 {
            self.id
        }

        pub fn user_id(&self) -> Uuid {
            self.user_id
        }

        pub fn pot_template_id(&self) -> i32 {
            self.pot_template_id
        }
    }

    #[derive(Insertable)]
    #[diesel(table_name = pot_template_users)]
    pub struct NewPotTemplateUser {
        user_id: Uuid,
        pot_template_id: i32
    }

    impl NewPotTemplateUser {
        pub fn new(user_id: Uuid, pot_template_id: i32) -> Self {
            Self { user_id, pot_template_id }
        }

        pub fn user_id(&self) -> Uuid {
            self.user_id
        }

        pub fn pot_template_id(&self) -> i32 {
            self.pot_template_id
        }
    }
}