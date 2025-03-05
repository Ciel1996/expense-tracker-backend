pub mod expense_service {
    use crate::{internal_error, not_found_error, ExpenseError};
    use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
    use diesel::associations::HasTable;
    use diesel::result::Error;
    use diesel_async::{AsyncConnection, RunQueryDsl};
    use diesel_async::scoped_futures::ScopedFutureExt;
    use expense_tracker_db::expenses::expenses::{Expense, NewExpense};
    use expense_tracker_db::schema::expense_splits::dsl::expense_splits;
    use expense_tracker_db::schema::expenses::dsl::expenses;
    use expense_tracker_db::schema::expenses::pot_id;
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

            let result = conn.transaction::<_, Error, _>(|conn| async move {
                let expense = diesel::insert_into(expenses)
                    .values(new_expense_clone)
                    .returning(Expense::as_returning())
                    .get_result::<Expense>(conn)
                    .await?;

                let splits =
                    NewExpenseSplit::splits_from_vector_with_id(splits, expense.id());

                let splits = diesel::insert_into(expense_splits)
                    .values(&splits)
                    .returning(Split::as_returning())
                    .get_results::<Split>(conn)
                    .await?;

                Ok((expense, splits))
            }.scope_boxed())
                .await
                .map_err(not_found_error)?;

            Ok(result)
        }

        /// Gets all expenses for the pot with the given target_pot_id.
        pub async fn get_expenses_by_pot_id(
            &self,
            target_pot_id : i32
        ) -> Result<Vec<Expense>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            let pot_expenses = expenses
                .select(pot_id.eq(target_pot_id))
                .get_results(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(pot_expenses)
        }

    }

    /// Creates a new ExpenseService with the given DbConnectionPool.
    pub fn new_service(pool: DbPool) -> ExpenseService {
        ExpenseService { db_pool: pool }
    }
}
