pub mod splits {
    use diesel::{Associations, Insertable, Queryable, Selectable};
    use serde::Deserialize;
    use crate::expenses::expenses::Expense;
    use crate::schema::expense_splits;

    /// This struct represents a split which is in turn part of an Expense
    /// but related to a user. The user is the one owing money the owner of
    /// the expense.
    #[derive(Selectable, Queryable, Associations, Clone)]
    #[diesel(belongs_to(Expense))]
    #[diesel(table_name = expense_splits)]
    pub struct Split {
        expense_id : i32,
        user_id : i32,
        amount : f64,
        is_paid: bool
    }

    impl Split {
        /// Constructor for a Split.
        pub fn new(
            expense_id : i32,
            user_id : i32,
            amount : f64,
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
        pub fn amount(&self) -> f64 {
            self.amount
        }

        /// Getter for is_paid.
        pub fn is_paid(&self) -> bool {
            self.is_paid
        }
    }

    /// Struct used to create a new Split in the db.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = expense_splits)]
    pub struct NewSplit {
        expense_id : i32,
        user_id : i32,
        amount : f64,
        is_paid: bool
    }

    impl NewSplit {
        pub fn new(expense_id: i32, user_id: i32, amount: f64, is_paid: bool) -> Self {
            Self { expense_id, user_id, amount, is_paid }
        }
    }

    /// Use this struct if you want to create a new Expense with splits.
    pub struct NewExpenseSplit {
        user_id : i32,
        amount : f64,
        is_paid: bool
    }

    impl NewExpenseSplit {
        pub fn new(user_id: i32, amount: f64, is_paid: bool) -> Self {
            Self { user_id, amount, is_paid }
        }

        /// Turns this NewExpenseSplit into a NewSplit with the given expense_id.
        /// Must be called AFTER the Expense has been created.
        pub fn with_id(&self, expense_id: i32) -> NewSplit {
            NewSplit::new(expense_id, self.user_id, self.amount, self.is_paid)
        }

        /// Takes in the given `Vec<NewExpenseSplit>` and adds the given expense_id to each
        /// element.
        pub fn splits_from_vector_with_id(from: Vec<Self>, expense_id: i32) -> Vec<NewSplit> {
            let mut destination = vec!();

            for without in from {
                destination.push(without.with_id(expense_id));
            }

            destination
        }
    }
}