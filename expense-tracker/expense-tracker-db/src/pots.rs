pub mod pots {
    use diesel::{Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};
    use crate::schema::pots;

    /// Represents a pot. A pot is an accumulation of expenses and is owned by a single
    /// user. A pot can be shared with multiple users. The users can leave a pot
    /// anytime.
    /// The owner of a pot needs to invite other users to participate in a pot.
    #[derive(Serialize, Selectable, Queryable)]
    pub struct Pot {
        id: i32,
        owner_id: i32,
        name: String,
        // expenses: Vec<Expense>,
        default_currency_id: i32,
    }

    /// This struct is used to create a new pot in the database.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = pots)]
    pub struct NewPot {
        owner_id: i32,
        name: String,
        // expenses: Vec<Expense>,
        default_currency_id: i32,
    }

    impl Pot {
        /// Constructor for Pot
        pub fn new(owner_id: i32, name: String, default_currency_id: i32) -> Pot {
            Pot {
                id: 1, // TODO: autoincrement this
                owner_id,
                name,
                // expenses: vec![],
                default_currency_id,
            }
        }

        // ///Adds an expense to the pot
        // pub fn add_expense(&mut self, expense: Expense) {
        //     self.expenses.push(expense);
        // }
        //
        // /// Removes the Expense with the given expense_id from this pot.
        // pub fn remove_expense(&mut self, expense_id: u64) {
        //     if self.expenses.is_empty() {
        //         return;
        //     }
        //
        //     let index = self
        //         .expenses
        //         .iter()
        //         .position(|x| *x.expense_id() == expense_id);
        //
        //     if let Some(index) = index {
        //         self.expenses.remove(index);
        //     }
        // }
        //
        // /// Calculates the sum of all unpaid Expenses from the given User's perspective.
        // pub fn get_open_sum(&self, user: User) -> f64 {
        //     let mut sum = 0.0;
        //
        //     for expense in self.expenses {
        //         if expense.owner_id() == user.user_id() {
        //             // if user is owner of the expense
        //             // we can subtract the amount of the expense
        //             // (other users owe the user money)
        //             sum -= expense.amount()
        //         } else {
        //             // if user is not the owner of the expense
        //             // we must add the amount of the expense
        //             // (we owe the other users)
        //             sum += expense.amount();
        //         }
        //     }
        //
        //     sum
        // }
    }
}