pub mod expense_service {
    use std::rc::Rc;
    use diesel::{RunQueryDsl, SelectableHelper};
    use expense_tracker_db::expenses::expenses::{Expense, NewExpense};
    use expense_tracker_db::pots::pots::Pot;
    use expense_tracker_db::schema::expenses::dsl::expenses;
    use expense_tracker_db::setup::DbConnectionPool;
    use expense_tracker_db::splits::splits::{NewSplit, Split};
    use crate::{internal_error, internal_error_str};

    /// Struct working with Expense related logic.
    pub struct ExpenseService {
        db_pool: DbConnectionPool,
    }

    impl ExpenseService {
        /// Creates a new Expense for the given Pot.
        pub async fn create_expense(
            &self,
            new_expense : NewExpense,
            splits : Vec<NewSplit>
        )
            -> Result<Expense, String> {
            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let new_expense_clone = new_expense.clone();

            // TODO: these two operations should be transactional
            let res = conn
                .interact(move |conn| {
                    diesel::insert_into(expenses)
                        .values(new_expense_clone)
                        .returning(Expense::as_returning())
                        .get_result::<Expense>(conn)
                })
                .await
                .map_err(internal_error)?
                .map_err(internal_error)?;

            let split_res = self.create_splits(&res, &splits)
                .await
                .map_err(internal_error_str)?;

            Ok(res)
        }

        /// Creates the splits for the given Expense.
        pub async fn create_splits(&self, expense : &Expense, splits : &Vec<NewSplit>)
            -> Result<Vec<Split>, String> {
            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let res = conn
                .interact(move |conn| {
                    diesel::insert_into(splits)
                        .values(splits)
                        .returning(Split::as_returning())
                        .get_results::<Split>(conn)
                })
                .await
                .map_err(internal_error)?
                .map_err(internal_error)?;

            Ok(res)
        }
    }


    /// Creates a new ExpenseService with the given DbConnectionPool.
    pub fn new_service(pool : DbConnectionPool) -> ExpenseService {
        ExpenseService {
            db_pool : pool,
        }
    }
}