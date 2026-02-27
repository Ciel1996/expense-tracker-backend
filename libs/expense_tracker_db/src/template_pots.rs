pub mod template_pots {
    use crate::ExpenseTrackerDBError; //marked as unused but required by DbEnum
    use crate::schema::pot_templates;
    use crate::schema::pot_template_users;
    use chrono::{DateTime, Utc};
    use diesel::sql_types::VarChar; //marked as unused but required by DbEnum
    use diesel::{AsExpression, FromSqlRow, Insertable, Queryable, Selectable};
    use diesel_enum::DbEnum;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use utoipa::ToSchema;

    /// An enumeration used to define how often a new pot should be created from the given template.
    #[derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        FromSqlRow,
        DbEnum,
        Serialize,
        Deserialize,
        AsExpression,
        ToSchema
    )]
    #[diesel(sql_type = VarChar)]
    #[diesel_enum(error_fn = ExpenseTrackerDBError::not_found)]
    #[diesel_enum(error_type = ExpenseTrackerDBError)]
    pub enum Occurrence {
        Once,
        Daily,
        Weekly,
        Monthly,
        Yearly,
    }

    #[derive(Clone, Serialize, Selectable, Queryable)]
    #[diesel(table_name = pot_templates)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct PotTemplate {
        id: i32,
        owner_id: Uuid,
        name: String,
        default_currency_id: i32,
        create_at: DateTime<Utc>,
        occurrence: Occurrence,
    }
    
    impl PotTemplate {
        pub fn new(
            id: i32,
            owner_id: Uuid,
            name: String,
            default_currency_id: i32,
            create_at: DateTime<Utc>,
            occurrence: Occurrence
        ) -> Self {
            Self {
                id,
                owner_id,
                name,
                default_currency_id,
                create_at,
                occurrence
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

        pub fn occurrence(&self) -> Occurrence {
            self.occurrence
        }
    }

    #[derive(Clone, Deserialize, Insertable)]
    #[diesel(table_name = pot_templates)]
    pub struct NewPotTemplate {
        owner_id: Uuid,
        name: String,
        default_currency_id: i32,
        create_at: DateTime<Utc>,
        occurrence: Occurrence,
    }

    impl NewPotTemplate {
        pub fn new(
            owner_id: Uuid, 
            name: String,
            default_currency_id: i32,
            create_at: DateTime<Utc>,
            occurrence: Occurrence
        ) -> Self {
            Self {
                owner_id,
                name,
                default_currency_id,
                create_at,
                occurrence,
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

        pub fn occurrence(&self) -> Occurrence {
            self.occurrence
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