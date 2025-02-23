pub mod expense_service {
    use crate::{internal_error, not_found_error, ExpenseError};
    use diesel::{SelectableHelper};
    use diesel_async::RunQueryDsl;
    use expense_tracker_db::expenses::expenses::{Expense, NewExpense};
    use expense_tracker_db::schema::expense_splits::dsl::expense_splits;
    use expense_tracker_db::schema::expenses::dsl::expenses;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::splits::splits::{NewExpenseSplit, Split};

    /// Struct working with Expense related logic.
    pub struct ExpenseService {
        db_pool: DbPool,
    }

    impl ExpenseService {
        /// Creates a new Expense for the given Pot.
        pub async fn create_expense(
            &self,
            new_expense: NewExpense,
            splits: Vec<NewExpenseSplit>,
        ) -> Result<(Expense, Vec<Split>), ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            let new_expense_clone = new_expense.clone();

            // TODO: TRANSACTIONOFY THIS!
            let expense = diesel::insert_into(expenses)
                .values(new_expense_clone)
                .returning(Expense::as_returning())
                .get_result::<Expense>(&mut conn)
                .await
                .map_err(not_found_error)?;

            let splits = NewExpenseSplit::splits_from_vector_with_id(splits, expense.id());

            let splits = diesel::insert_into(expense_splits)
                .values(&splits)
                .returning(Split::as_returning())
                .get_results::<Split>(&mut conn)
                .await
                .map_err(not_found_error)?;

            Ok((expense, splits))
        }
    }

    /// Creates a new ExpenseService with the given DbConnectionPool.
    pub fn new_service(pool: DbPool) -> ExpenseService {
        ExpenseService { db_pool: pool }
    }
}
