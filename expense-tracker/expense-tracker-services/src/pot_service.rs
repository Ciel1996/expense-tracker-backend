pub mod pot_service {
    use crate::currency_service::currency_service;
    use crate::currency_service::currency_service::CurrencyService;
    use crate::{check_error, internal_error, not_found_error, ExpenseError};
    use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::pots::pots::{NewPot, Pot};
    use expense_tracker_db::schema::pots::dsl::pots;
    use expense_tracker_db::schema::pots::{id as pots_id, id, owner_id};
    use expense_tracker_db::setup::DbPool;

    /// A service offering interfaces related to Pots.
    #[derive(Clone)]
    pub struct PotService {
        db_pool: DbPool,
        currency_service: CurrencyService,
    }

    impl PotService {
        /// Creates a pot with the given NewPot.
        pub async fn create_pot(&self, new_pot: NewPot) -> Result<(Pot, Currency), ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            let loaded_pot_currency_id = new_pot.default_currency_id().clone();

            let currency = self
                .currency_service
                .get_currency_by_id(loaded_pot_currency_id)
                .await
                .map_err(check_error)?;

            let pot = diesel::insert_into(pots)
                .values(new_pot)
                .returning(Pot::as_returning())
                .get_result::<Pot>(&mut conn)
                .await
                .map_err(not_found_error)?;

            Ok((pot, currency))
        }

        /// Gets a Vector of all Pots.
        pub async fn get_pots(&self, user_uuid : Uuid) -> Result<Vec<Pot>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // let pots_with_user : Vec<i32> = pots_to_users
            //     .filter(user_id.eq(user_uuid))
            //     .select(pot_id)
            //     .load(&mut conn)
            //     .await
            //     .map_err(|_| Vec::new())?;

            let loaded_pots = pots
                .filter(owner_id.eq(user_uuid))
                .select(Pot::as_select())
                .load(&mut conn)
                .await
                .map_err(not_found_error)?;

            Ok(loaded_pots)
        }

        /// Gets a Pot with the given id if it exists.
        pub async fn get_pot_by_id(
            &self,
            to_search: i32,
            requester_id : Uuid) -> Result<Pot, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            pots.filter(owner_id.eq(requester_id).and(pots_id.eq(to_search)))
                .get_result::<Pot>(&mut conn)
                .await
                .map_err(not_found_error)
        }
    }

    /// Creates a new PotService with the given DbConnectionPool.
    pub fn new_service(pool: DbPool) -> PotService {
        PotService {
            db_pool: pool.clone(),
            currency_service: currency_service::new_service(pool.clone()),
        }
    }
}
