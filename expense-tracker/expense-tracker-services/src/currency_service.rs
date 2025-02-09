pub mod currency_service {
    use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, ExpressionMethods};
    use expense_tracker_db::currencies::currencies::{Currency, NewCurrency};
    use expense_tracker_db::schema::currencies::dsl::currencies;
    use expense_tracker_db::setup::DbConnectionPool;
    use expense_tracker_db::schema::currencies::{id, symbol};
    use crate::{internal_error, internal_error_str};

    /// The service responsible for interacting with Currency related logic.
    #[derive(Clone)]
    pub struct CurrencyService {
        db_pool: DbConnectionPool,
    }

    impl CurrencyService {
        /// Gets a Currency for the given symbol.
        /// Returns either None if no Currency for the symbol could be found
        /// or the Currency that is related to the given symbol.
        pub async fn get_currency_by_symbol(&self, currency_symbol : String)
            -> Result<Option<Currency>, String> {
            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let res = conn
                .interact(|conn| currencies
                    .filter(symbol.eq(currency_symbol))
                    .first::<Currency>(conn)
                )
                .await.map_err(internal_error)?;

            match res {
                Ok(c) => Ok(Some(c)),
                _ => Ok(None),
            }
        }

        /// Gets a Currency by the given id. If no Currency with the given id
        /// could be found, returns Ok(None).
        pub async fn get_currency_by_id(&self, to_search: i32)
            -> Result<Option<Currency>, String>
        {
            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let res = conn
                .interact(move |conn| currencies
                    .filter(id.eq(to_search))
                    .first::<Currency>(conn)
                )
                .await.map_err(internal_error)?;

            match res {
                Ok(c) => Ok(Some(c)),
                _ => Ok(None)
            }
        }

        /// Checks if the a new Currency can be created with the information provided by the given
        /// NewCurrency. If so, it is created and returned. Otherwise an error will be returned.
        pub async fn create_currency(&self, new_currency: NewCurrency)
            -> Result<Currency, String> {
            let existing_currency = self
                .get_currency_by_symbol(new_currency.symbol().to_string())
                .await
                .map_err(internal_error_str)?;

            if existing_currency.is_some() {
                return Err(
                    format!(
                        "There is already a currency with symbol {}!",
                        new_currency.symbol().to_string()
                    )
                );
            }

            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let res = conn
                .interact(move |conn| {
                    diesel::insert_into(currencies)
                        .values(new_currency)
                        .returning(Currency::as_returning())
                        .get_result::<Currency>(conn)
                })
                .await
                .map_err(internal_error)?
                .map_err(internal_error)?;

            Ok(res)
        }

        /// Gets all currencies.
        pub async fn get_currencies(&self) -> Result<Vec<Currency>, String> {
            let conn  = self.db_pool.get().await.map_err(internal_error)?;

            let loaded_currencies = conn
                .interact(|conn| currencies
                    .select(Currency::as_select())
                    .load::<Currency>(conn)
                )
                .await
                .map_err(internal_error)?
                .map_err(internal_error)?;

            Ok(loaded_currencies)
        }
    }

    /// Creates a new instance of CurrencyService.
    pub fn new_service(pool : DbConnectionPool) -> CurrencyService {
        CurrencyService{
            db_pool : pool
        }
    }
}