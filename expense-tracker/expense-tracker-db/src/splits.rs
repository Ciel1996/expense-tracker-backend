pub mod splits {
    use diesel::{Associations, Insertable, PgConnection, Queryable, Selectable};
    use serde::{Deserialize, Serialize};

    /// This struct represents a split which is in turn part of an Expense
    /// but related to a user. The user is the one owing money the owner of
    /// the expense.
    #[derive(Serialize, Selectable, Queryable, Associations)]
    #[diesel(belongs_to(Expense))]
    pub struct Split {
        expense_id : i32,
        user_id : i32,
        amount : i32,
        is_paid: bool
    }

    impl Split {
        /// Constructor for a Split.
        pub fn new(
            expense_id : i32,
            user_id : i32,
            amount : i32,
            is_paid: bool
        ) -> Self {
            Split {
                expense_id,
                user_id,
                amount,
                is_paid
            }
        }

        /// Getter for expense_id.
        pub fn expense_id(&self) -> i32 {
            self.expense_id
        }

        /// Getter for user_id.
        pub fn user_id(&self) -> i32 {
            self.user_id
        }

        /// Getter for amount.
        pub fn amount(&self) -> i32 {
            self.amount
        }

        /// Getter for is_paid.
        pub fn is_paid(&self) -> bool {
            self.is_paid
        }
    }

    /// Struct used to create a new Split in the db.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = splits)]
    pub struct NewSplit {
        expense_id : i32,
        user_id : i32,
        amount : i32,
        is_paid: bool
    }
}