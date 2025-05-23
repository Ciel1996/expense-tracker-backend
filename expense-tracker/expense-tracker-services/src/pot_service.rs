pub mod pot_service {
    use crate::currency_service::currency_service;
    use crate::currency_service::currency_service::CurrencyService;
    use crate::{check_error, internal_error, not_found_error, ExpenseError};
    use diesel::{BoolExpressionMethods, ExpressionMethods, Insertable, QueryDsl, SelectableHelper};
    use diesel::dsl::{count_star, CountStar};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::pots::pots::{NewPot, Pot, PotToUser};
    use expense_tracker_db::schema::pots::dsl::pots;
    use expense_tracker_db::schema::pots::{id as pots_id, id, owner_id};
    use expense_tracker_db::schema::pots_to_users::dsl::pots_to_users;
    use expense_tracker_db::schema::pots_to_users::{pot_id, user_id};
    use expense_tracker_db::setup::DbPool;
    use crate::ExpenseError::Conflict;

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

        /// Adds `new_user_id` to the pot with the given `the_pot_id` if the user with the `requester_id`
        /// is the owner of that given pot.
        pub async fn add_user_to_pot(
            &self,
            pot_to_user : PotToUser,
            requester_id : Uuid) -> Result<bool, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let the_pot_id = pot_to_user.pot_id();

            let get_pot_by_id_and_owner = pots
                .filter(id.eq(the_pot_id).and(owner_id.eq(requester_id)))
                .select(Pot::as_select())
                .load(&mut conn)
                .await
                .map_err(internal_error)?;

            if get_pot_by_id_and_owner.is_empty() {
                return Ok(false);
            }

            let new_user_id = pot_to_user.user_id();

            let pots_containing_user = pots_to_users
                .filter(pot_id.eq(the_pot_id).and(user_id.eq(new_user_id)))
                .count()
                .get_result::<i64>(&mut conn)
                .await
                .map_err(internal_error)?;

            if pots_containing_user > 0 {
                return Err(
                    Conflict(
                        format!("User {} was previously added to pot {}",
                                new_user_id, the_pot_id
                        )));
            }

            let result = diesel::insert_into(pots_to_users)
                .values(pot_to_user)
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(result > 0)
        }

        /// Gets a Vector of all Pots.
        pub async fn get_pots(&self, user_uuid : Uuid) -> Result<Vec<Pot>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            let pots_with_user : Vec<i32> = pots_to_users
                .filter(user_id.eq(user_uuid))
                .select(pot_id)
                .load(&mut conn)
                .await
                .unwrap_or_else(|_| Vec::new());

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
