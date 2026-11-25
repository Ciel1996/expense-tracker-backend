pub mod expenses {
    use crate::pots::pots::Pot;
    use crate::schema::expenses;
    use diesel::{Associations, Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    /// An expense is an amount of money paid, as well as associated information by a user.
    /// An expense can either be paid or unpaid. Unpaid expenses should be considered for
    /// the sum other users have to pay.
    #[derive(Serialize, Selectable, Queryable, Associations, Clone)]
    #[diesel(belongs_to(Pot))]
    pub struct Expense {
        id: i32,
        pot_id: i32,
        owner_id: Uuid,
        description: String,
        currency_id: i32,
        expense_date: chrono::DateTime<chrono::Utc>
    }

    /// This struct is used to create a new expense in the database.
    #[derive(Deserialize, Insertable, Clone)]
    #[diesel(table_name = expenses)]
    pub struct NewExpense {
        owner_id: Uuid,
        pot_id: i32,
        description: String,
        currency_id: i32,
        expense_date: Option<chrono::DateTime<chrono::Utc>>
    }

    impl Expense {
        /// Constructor for Expense
        pub fn new(
            id: i32,
            pot_id: i32,
            owner_id: Uuid,
            description: String,
            currency_id: i32,
            expense_date: Option<chrono::DateTime<chrono::Utc>>
        ) -> Expense {
            Expense {
                id,
                pot_id,
                owner_id,
                description,
                currency_id,
                expense_date: expense_date.unwrap_or_else(|| chrono::Utc::now()),
            }
        }

        /// Getter for pot_id.
        pub fn pot_id(&self) -> i32 {
            self.pot_id
        }

        /// Getter for description
        pub fn description(&self) -> &str {
            &self.description
        }

        /// Getter for owner_id
        pub fn owner_id(&self) -> Uuid {
            self.owner_id
        }

        /// Getter for id
        pub fn id(&self) -> i32 {
            self.id
        }

        pub fn currency_id(&self) -> i32 {
            self.currency_id
        }

        pub fn expense_date(&self) -> chrono::DateTime<chrono::Utc> {
            self.expense_date
        }
    }

    impl NewExpense {
        pub fn new(
            owner_id: Uuid,
            pot_id: i32,
            description: String,
            currency_id: i32,
            expense_date: Option<chrono::DateTime<chrono::Utc>>) -> Self {
            Self {
                owner_id,
                pot_id,
                description,
                currency_id,
                expense_date
            }
        }
    }
}
