pub mod pot_service {
    use std::error::Error;
    use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, ExpressionMethods};
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::pots::pots::{NewPot, Pot};
    use expense_tracker_db::setup::DbConnectionPool;
    use expense_tracker_db::schema::pots::dsl::pots;
    use crate::currency_service::currency_service;
    use crate::currency_service::currency_service::CurrencyService;
    use expense_tracker_db::schema::pots::id as pots_id;
    use crate::{check_error, internal_error, not_found_error, ExpenseError};

    /// A service offering interfaces related to Pots.
    #[derive(Clone)]
    pub struct PotService {
        db_pool : DbConnectionPool,
        currency_service : CurrencyService
    }

    impl PotService {
        /// Creates a pot with the given NewPot.
        pub async fn create_pot(
            &self,
            new_pot : NewPot
        ) -> Result<(Pot, Currency), ExpenseError> {
            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let loaded_pot_currency_id = new_pot.default_currency_id().clone();

            let currency = self.currency_service
                .get_currency_by_id(loaded_pot_currency_id)
                .await
                .map_err(check_error)?;

            let res = conn
                .interact(move |conn| {
                    diesel::insert_into(pots)
                        .values(new_pot)
                        .returning(Pot::as_returning())
                        .get_result::<Pot>(conn)
                })
                .await
                .map_err(internal_error)?
                .map_err(not_found_error)?;


            Ok((res, currency))
        }

        /// Gets a Vector of all Pots.
        pub async fn get_pots(
            &self
        ) -> Result<Vec<Pot>, ExpenseError> {
            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let loaded_pots = conn
                .interact(|conn| pots.select(Pot::as_select())
                    .load::<Pot>(conn))
                .await
                .map_err(internal_error)?
                .map_err(not_found_error)?;

            Ok(loaded_pots)
        }

        /// Gets a Pot with the given id if it exists.
        pub async fn get_pot_by_id(
            &self,
            to_search : i32
        ) -> Result<Pot, ExpenseError> {
            let conn = self.db_pool.get()
                .await
                .map_err(internal_error)?;

           conn
                .interact(move |conn| pots
                    .filter(pots_id.eq(to_search))
                    .first::<Pot>(conn)
                )
                .await
                .map_err(internal_error)?
                .map_err(not_found_error)
        }
    }

    /// Creates a new PotService with the given DbConnectionPool.
    pub fn new_service(pool : DbConnectionPool)
        -> PotService {
        PotService {
            db_pool : pool.clone(),
            currency_service: currency_service::new_service(pool.clone())
        }
    }
}