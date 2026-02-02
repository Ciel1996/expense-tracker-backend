pub mod splits {
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};
    use crate::expenses::expenses::Expense;
    use crate::schema::expense_splits;
    use diesel::{Associations, Insertable, Queryable, Selectable};
    use serde::Deserialize;
    use uuid::Uuid;

    /// This struct represents a split which is in turn part of an Expense
    /// but related to a user. The user is the one owing money the owner of
    /// the expense.
    #[derive(Selectable, Queryable, Associations, Clone)]
    #[diesel(belongs_to(Expense))]
    #[diesel(table_name = expense_splits)]
    pub struct Split {
        expense_id: i32,
        user_id: Uuid,
        amount: f64,
        is_paid: bool,
    }

    impl Split {
        /// Constructor for a Split.
        pub fn new(expense_id: i32, user_id: Uuid, amount: f64, is_paid: bool) -> Self {
            Split {
                expense_id,
                user_id,
                amount,
                is_paid,
            }
        }

        /// Getter for expense_id.
        pub fn expense_id(&self) -> i32 {
            self.expense_id
        }

        /// Getter for user_id.
        pub fn user_id(&self) -> Uuid {
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
        expense_id: i32,
        user_id: Uuid,
        amount: f64,
        is_paid: bool,
    }

    impl NewSplit {
        pub fn new(expense_id: i32, user_id: Uuid, amount: f64, is_paid: bool) -> Self {
            Self {
                expense_id,
                user_id,
                amount,
                is_paid,
            }
        }
    }

    impl PartialEq<Self> for NewSplit {
        fn eq(&self, other: &Self) -> bool {
            self.user_id == other.user_id && self.expense_id == other.expense_id
        }
    }

    impl Eq for NewSplit {}

    impl Hash for NewSplit {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.user_id.hash(state);
            self.expense_id.hash(state);
        }
    }

    /// Use this struct if you want to create a new Expense with splits.
    pub struct NewExpenseSplit {
        user_id: Uuid,
        amount: f64,
        is_paid: bool,
    }

    impl NewExpenseSplit {
        pub fn new(user_id: Uuid, amount: f64) -> Self {
            Self {
                user_id,
                amount,
                is_paid: false
            }
        }

        pub fn set_payment_status(&mut self, is_paid : bool) {
            self.is_paid = is_paid
        }

        /// Turns this NewExpenseSplit into a NewSplit with the given expense_id.
        /// Must be called AFTER the Expense has been created.
        pub fn with_id(&self, expense_id: i32) -> NewSplit {
            NewSplit::new(expense_id, self.user_id, self.amount, self.is_paid)
        }

        /// Takes in the given `Vec<NewExpenseSplit>` and adds the given expense's id to each
        /// element.
        pub fn splits_from_vector_with_id(from: Vec<Self>, expense: &Expense) -> Vec<NewSplit> {
            let mut destination = vec![];

            let expense_id = expense.id();

            for mut without in from {
                without.set_payment_status(expense.owner_id() == without.user_id);
                destination.push(without.with_id(expense_id));
            }

            destination
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        }
    }
}
