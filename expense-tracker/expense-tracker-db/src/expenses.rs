pub mod expenses {
    use diesel::{Associations, Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};
    use crate::schema::expenses;
    use crate::pots::pots::Pot;

    /// An expense is an amount of money paid, as well as associated information by a user.
    /// An expense can either be paid or unpaid. Unpaid expenses should be considered for
    /// the sum other users have to pay.
    #[derive(Serialize, Selectable, Queryable, Associations)]
    #[diesel(belongs_to(Pot))]
    pub struct Expense {
        id: i32,
        pot_id: i32,
        owner_id: i32,
        description: String,
        amount: f64,
        currency_id: i32,
        is_paid: bool,
    }

    /// This struct is used to create a new expense in the database.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = expenses)]
    pub struct NewExpense {
        owner_id: i32,
        description: String,
        amount: f64,
        currency_id: i32,
        is_paid: bool,
    }

    impl Expense {
        /// Constructor for Expense
        pub fn new(
            id: i32,
            pot_id: i32,
            owner_id: i32,
            description: String,
            amount: f64,
            currency_id: i32,
            is_paid: bool,
        ) -> Expense {
            Expense {
                id,
                pot_id,
                owner_id,
                description,
                amount,
                currency_id,
                is_paid,
            }
        }

        /// Getter for pot_id.
        pub fn pot_id(&self) -> i32 {
            self.pot_id
        }

        /// Marks this Expense as paid.
        pub fn pay(&mut self) {
            self.is_paid = true
        }

        /// Getter for amount
        pub fn amount(&self) -> f64 {
            self.amount
        }

        /// Getter for is_paid
        pub fn is_paid(&self) -> bool {
            self.is_paid
        }

        /// Getter for description
        pub fn description(&self) -> &str {
            &self.description
        }

        /// Getter for owner_id
        pub fn owner_id(&self) -> i32 {
            self.owner_id
        }

        /// Getter for id
        pub fn id(&self) -> i32 {
            self.id
        }
    }
}