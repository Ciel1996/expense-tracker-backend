pub mod expense_service {
    use diesel::{RunQueryDsl, SelectableHelper};
    use expense_tracker_db::expenses::expenses::{Expense, NewExpense};
    use expense_tracker_db::schema::expense_splits::dsl::expense_splits;
    use expense_tracker_db::schema::expenses::dsl::expenses;
    use expense_tracker_db::setup::DbConnectionPool;
    use expense_tracker_db::splits::splits::{NewExpenseSplit, NewSplit, Split};
    use crate::internal_error;

    /// Struct working with Expense related logic.
    pub struct ExpenseService {
        db_pool: DbConnectionPool,
    }

    impl ExpenseService {
        /// Creates a new Expense for the given Pot.
        pub async fn create_expense(
            &self,
            new_expense : NewExpense,
            splits : Vec<NewExpenseSplit>
        )
            -> Result<(Expense, Vec<Split>), String> {
            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let new_expense_clone = new_expense.clone();

            // TODO: find out how to run a transaction with diesel + deadpool or replace deadpool
            let expense = conn
                .interact(move |conn| {
                    diesel::insert_into(expenses)
                        .values(new_expense_clone)
                        .returning(Expense::as_returning())
                        .get_result::<Expense>(conn)
                })
                .await
                .map_err(internal_error)?
                .map_err(internal_error)?;

            let splits = NewExpenseSplit::splits_from_vector_with_id(
                    splits,
                    expense.id()
            );

            let splits = conn
                .interact(move |conn| {
                    // TODO: transaction
                    diesel::insert_into(expense_splits)
                        .values(&splits)
                        .returning(Split::as_returning())
                        .get_results::<Split>(conn)
                })
                .await
                .map_err(internal_error)?
                .map_err(internal_error)?;

            Ok((expense, splits))
        }
    }

    /// Creates a new ExpenseService with the given DbConnectionPool.
    pub fn new_service(pool : DbConnectionPool) -> ExpenseService {
        ExpenseService {
            db_pool : pool,
        }
    }
}