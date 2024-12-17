// pub mod db {
//     /// Currency represents a supported real-world currency.
//     enum Currency {
//         SwissFranc,
//         Euro,
//         UsDollar,
//     }
//
//     pub mod expenses {
//         use diesel::{Insertable, Queryable, Selectable};
//         use serde::Serialize;
//         use crate::db::Currency;
//         use crate::db::users::User;
//
//         /// An expense is an amount of money paid, as well as associated information by a user.
//         /// An expense can either be paid or unpaid. Unpaid expenses should be considered for
//         /// the sum other users have to pay.
//         #[derive(Serialize, Selectable, Queryable, Insertable)]
//         pub struct Expense {
//             owner_id : u64,
//             expense_id : u64,
//             description : String,
//             amount : f64,
//             currency : Currency,
//             is_paid : bool,
//         }
//
//         impl Expense {
//             /// Constructor for Expense
//             pub fn new(user : User, description : String, amount : f64, currency: Currency) -> Expense {
//                 Expense {
//                     owner_id: user.user_id(),
//                     expense_id: 1, // TODO: autoincrement
//                     description,
//                     amount,
//                     currency,
//                     is_paid: false
//                 }
//             }
//
//             /// Marks this Expense as paid.
//             pub fn pay(&mut self) {
//                 self.is_paid = true
//             }
//
//             /// Getter for amount
//             pub fn amount(&self) -> f64 {
//                 self.amount
//             }
//
//             /// Getter for is_paid
//             pub fn is_paid(&self) -> bool {
//                 self.is_paid
//             }
//
//             /// Getter for description
//             pub fn description(&self) -> &str {
//                 &self.description
//             }
//
//             /// Getter for owner_id
//             pub fn owner_id(&self) -> u64 {
//                 self.owner_id
//             }
//
//             /// Getter for expense_id
//             pub fn expense_id(&self) -> u64 {
//                 self.expense_id
//             }
//         }
//     }
//
//     pub mod pots {
//         use diesel::{Insertable, Queryable, Selectable};
//         use serde::Serialize;
//         use crate::db::Currency;
//         use crate::db::expenses::Expense;
//         use crate::db::users::User;
//
//         /// Represents a pot. A pot is an accumulation of expenses and is owned by a single
//         /// user. A pot can be shared with multiple users. The users can leave a pot
//         /// anytime.
//         /// The owner of a pot need to invite other users to participate in a pot.
//         #[derive(Serialize, Selectable, Queryable, Insertable)]
//         pub struct Pot {
//             owner_id : u64,
//             pot_id: u64,
//             name : String,
//             expenses : Vec<Expense>,
//             default_currency : Currency
//         }
//
//         impl Pot {
//             /// Constructor for Pot
//             pub fn new(user: User, name : String, default_currency : Currency) -> Pot {
//                 Pot {
//                     owner_id: user.user_id(),
//                     pot_id: 1, // TODO: autoincrement this
//                     name,
//                     expenses: vec![],
//                     default_currency
//                 }
//             }
//
//             /// Adds an expense to the pot
//             pub fn add_expense(&mut self, expense : Expense) {
//                 self.expenses.push(expense);
//             }
//
//             /// Removes the Expense with the given expense_id from this pot.
//             pub fn remove_expense(&mut self, expense_id : u64) {
//                 if self.expenses.is_empty() {
//                     return;
//                 }
//
//                 let index = self.expenses
//                     .iter().position(|x| *x.expense_id() == expense_id);
//
//                 if let Some(index) = index {
//                     self.expenses.remove(index);
//                 }
//             }
//
//             /// Calculates the sum of all unpaid Expenses from the given User's perspective.
//             pub fn get_open_sum(&self, user : User) -> f64 {
//                 let mut sum = 0.0;
//
//                 for expense in self.expenses {
//                     if expense.owner_id() == user.user_id() {
//                         // if user is owner of the expense
//                         // we can subtract the amount of the expense
//                         // (other users owe the user money)
//                         sum -= expense.amount()
//                     } else {
//                         // if user is not the owner of the expense
//                         // we must add the amount of the expense
//                         // (we owe the other users)
//                         sum += expense.amount();
//                     }
//                 }
//
//                 sum
//             }
//         }
//     }
//
    pub mod users {
        use diesel::{table, Insertable, Queryable, Selectable};
        use serde::{Deserialize, Serialize};

        // normally part of the generated schema.rs file
         table! {
            users(id) {
                id -> Integer,
                name -> Text
            }
        }

        /// Defines a user of the systems.
        /// A user can hold pots, expenses and so on.
        #[derive(Serialize, Selectable, Queryable)]
        pub struct User {
            id : i32,
            name : String
        }

        impl User {
            /// Constructor for User
            pub fn new(id : i32, name : String) -> User {
                User {
                    id,
                    name
                }
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
            name : String
        }

        impl NewUser {
            pub fn new(name : String) -> NewUser {
                NewUser {
                    name
                }
            }
        }
    }
// }

pub mod setup {
    use deadpool_diesel::Pool;
    use deadpool_diesel::postgres::{Manager, Object};
    use diesel::PgConnection;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    /// The exact type of the DbPool in this application.
    pub type DbPool = deadpool_diesel::postgres::Pool;

    /// The exact type of the connection pool that is used in this application.
    pub type DbConnectionPool = Pool<Manager, Object>;

    const MIGRATIONS : EmbeddedMigrations = embed_migrations!("../migrations/");

    /// Sets up the db for the application.
    pub async fn setup_db() -> DbConnectionPool {
        // TODO: load from config
        let db_string: String =
            String::from("postgres://admin:localpassword@localhost/expense-tracker");

        let manager
            = Manager::new(db_string, deadpool_diesel::Runtime::Tokio1);

        let pool = Pool::builder(manager).build().unwrap();

        run_migrations(&pool).await;

        pool
    }

    /// Runs the migrations on server startup.
    async fn run_migrations(pool: &Pool<Manager, Object>) {
        let conn = pool.get().await.unwrap();
        conn
            .interact(|conn|
                conn.run_pending_migrations(MIGRATIONS)
                    .map(|_|()))
            .await
            .unwrap()
            .unwrap();
    }
}