pub mod expense_service {
    use crate::{internal_error, not_found_error, ExpenseError};
    use diesel::{QueryDsl, SelectableHelper, ExpressionMethods};
    use diesel::result::Error;
    use diesel_async::{AsyncConnection, RunQueryDsl};
    use diesel_async::scoped_futures::ScopedFutureExt;
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::expenses::expenses::{Expense, NewExpense};
    use expense_tracker_db::schema::currencies::dsl::currencies;
    use expense_tracker_db::schema::expense_splits::dsl::expense_splits;
    use expense_tracker_db::schema::expense_splits::expense_id as split_expense_id;
    use expense_tracker_db::schema::expenses::dsl::expenses;
    use expense_tracker_db::schema::expenses::{id as expense_id, pot_id as expense_pot_id};
    use expense_tracker_db::schema::currencies::id as currencies_id;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::splits::splits::{NewExpenseSplit, Split};
    use crate::currency_service::currency_service;
    use crate::currency_service::currency_service::CurrencyService;

    // #[derive(Queryable)]
    // pub struct JoinedExpenseCurrency {
    //     expense_id: i32,
    //     expense_owner_id: i32,
    //     expense_pot_id: i32,
    //     expense_description: String,
    //     expense_currency_id: i32,
    //     currency_id: i32,
    //     currency_symbol: String,
    //     currency_name: String
    // }

    /// Represents a joined `Expense`, with a `Vec<Split>` and a `Currency`.
    pub type JoinedExpense = (Expense, Vec<Split>, Currency);

    /// Struct working with Expense related logic.
    pub struct ExpenseService {
        db_pool: DbPool,
        currency_service: CurrencyService,
    }

    impl ExpenseService {
        /// Creates a new Expense for the given Pot.
        pub async fn create_expense(
            &self,
            new_expense: NewExpense,
            splits: Vec<NewExpenseSplit>,
        ) -> Result<(Expense, Vec<Split>, Currency), ExpenseError> {
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

                let currency = self
                    .currency_service
                    .get_currency_by_id(expense.currency_id())
                    .await?;

                Ok((expense, splits, currency))
            }.scope_boxed())
                .await
                .map_err(not_found_error)?;

            Ok(result)
        }

        // TODO: this might require a specific ExpenseAPI!!
        /// Gets a single expense with all associated data by the given id.
        pub async fn get_expense_by_id(&self, target_id : i32)
            -> Result<JoinedExpense, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // let result = expenses
            //     .inner_join(currencies)
            //     .filter(expense_id.eq(target_id))
            //     .first::<JoinedExpenseCurrency>(&mut conn)
            //     .await
            //     .map_err(internal_error)?;

            // I'm just not able to make a join work with diesel and bb8.
            // I have to think about it a bit longer.
            let expense = expenses
                .filter(expense_id.eq(target_id))
                .get_result::<Expense>(&mut conn)
                .await
                .map_err(not_found_error)?;

            let splits = expense_splits
                .filter(split_expense_id.eq(target_id))
                .get_results::<Split>(&mut conn)
                .await
                .map_err(not_found_error)?;

            let currency = currencies
                .filter(currencies_id.eq(expense.currency_id()))
                .get_result::<Currency>(&mut conn)
                .await
                .map_err(not_found_error)?;

            Ok((expense, splits, currency))
        }

        /// Gets all expenses for the pot with the given target_pot_id.
        pub async fn get_expenses_by_pot_id(
            &self,
            target_pot_id : i32
        ) -> Result<Vec<JoinedExpense>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // TODO: doing this in a single db query doesn't work as easily as expected! Figure this out!
            // TODO: this will break with more and more db entries!! This has to be addressed asap! We could circumvent this a tiny bit by only selecting unpaid expenses, however this is currently not implemented!
            // let result = expenses
            //     .filter(expense_pot_id.eq(target_pot_id))
            //     .inner_join(expense_splits)
            //     .inner_join(currencies)
            //     .get_results::<JoinedExpense>(&mut conn)
            //     .await
            //     .map_err(internal_error)?;

            let pot_expenses = expenses
                .filter(expense_pot_id.eq(target_pot_id))
                .get_results::<Expense>(&mut conn)
                .await
                .map_err(internal_error)?;

            let all_currencies = currencies
                .get_results::<Currency>(&mut conn)
                .await
                .map_err(internal_error)?;

            let mut result : Vec<JoinedExpense> = vec!();

            for expense in pot_expenses {
                let currency = all_currencies
                    .iter()
                    .filter(|c| c.id() == expense.currency_id()).next();

                if let Some(currency) = currency {
                    let splits = expense_splits
                        .filter(split_expense_id.eq(expense.id()))
                        .get_results::<Split>(&mut conn)
                        .await
                        .map_err(internal_error)?;

                    result.push((expense, splits, (*currency).clone()))
                }

            }

            Ok(result)
        }
    }

    /// Creates a new ExpenseService with the given DbConnectionPool.
    pub fn new_service(pool: DbPool) -> ExpenseService {
        ExpenseService {
            db_pool: pool.clone(),
            currency_service: currency_service::new_service(pool.clone())
        }
    }
}
