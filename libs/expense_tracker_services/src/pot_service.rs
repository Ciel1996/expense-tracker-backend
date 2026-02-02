pub mod pot_service {
    use crate::currency_service::currency_service;
    use crate::currency_service::currency_service::CurrencyService;
    use crate::expense_service::expense_service;
    use crate::expense_service::expense_service::ExpenseService;
    use crate::ExpenseError::{Conflict, Forbidden, NotFound};
    use crate::{check_error, internal_error, not_found_error, ExpenseError};
    use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::pots::pots::{NewPot, Pot, PotToUser};
    use expense_tracker_db::schema::pots::dsl::pots;
    use expense_tracker_db::schema::pots::{id as pots_id, id, owner_id};
    use expense_tracker_db::schema::pots_to_users::dsl::pots_to_users;
    use expense_tracker_db::schema::pots_to_users::{pot_id, user_id};
    use expense_tracker_db::schema::users::dsl::users;
    use expense_tracker_db::schema::users::id as db_user_id;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::users::users::User;
    use uuid::Uuid;
    use log::error;

    /// A service offering interfaces related to Pots.
    #[derive(Clone)]
    pub struct PotService {
        db_pool: DbPool,
        currency_service: CurrencyService,
        expense_service: ExpenseService,
    }

    impl PotService {
        /// Creates a pot with the given NewPot.
        pub async fn create_pot(
            &self,
            new_pot: NewPot,
        ) -> Result<(Pot, Currency, Vec<User>), ExpenseError> {
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

            let pot_users = users
                .filter(db_user_id.eq(pot.owner_id()))
                .select(User::as_returning())
                .load(&mut conn)
                .await
                .map_err(internal_error)?;

            let pot_to_user = PotToUser::new(pot.id().clone(), pot.owner_id().clone());
            let result = self.add_user_to_joined_table(pot_to_user).await?;

            if result > 0 {
                error!("Could not add user '{}' to joined table", pot.owner_id().clone());
            }

            Ok((pot, currency, pot_users))
        }

        /// Adds `new_user_id` to the pot with the given `the_pot_id` if the user with the `requester_id`
        /// is the owner of that given pot.
        pub async fn add_user_to_pot(
            &self,
            pot_to_user: PotToUser,
            requester_id: Uuid,
        ) -> Result<bool, ExpenseError> {
            let the_pot_id = pot_to_user.pot_id();

            let get_pot_by_id_and_owner =
                Self::get_pot_by_id_and_owner(self, the_pot_id, requester_id).await?;

            if get_pot_by_id_and_owner.is_empty() {
                return Ok(false);
            }

            let new_user_id = pot_to_user.user_id();

            let pots_containing_user =
                Self::get_pots_containing_user(self, the_pot_id, new_user_id).await?;

            if pots_containing_user > 0 {
                return Err(Conflict(format!(
                    "User {} was previously added to pot {}",
                    new_user_id, the_pot_id
                )));
            }

            let result = self.add_user_to_joined_table(pot_to_user).await?;

            Ok(result > 0)
        }

        async fn add_user_to_joined_table(
            &self,
            pot_to_user: PotToUser
        ) -> Result<usize, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            diesel::insert_into(pots_to_users)
                .values(pot_to_user)
                .execute(&mut conn)
                .await
                .map_err(internal_error)
        }

        /// Removes `user_id` from the pot with the given `the_pot_id` if the user with the `requester_id`
        /// is the owner of that given pot.
        pub async fn remove_user_from_pot(
            &self,
            pot_to_user: PotToUser,
            requester_id: Uuid,
        ) -> Result<bool, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let the_pot_id = pot_to_user.pot_id();

            let get_pot_by_id_and_owner =
                Self::get_pot_by_id_and_owner(self, the_pot_id, requester_id).await?;

            if get_pot_by_id_and_owner.is_empty() {
                return Ok(false);
            }

            let to_delete_user_id = pot_to_user.user_id();

            let pots_containing_user =
                Self::get_pots_containing_user(self, the_pot_id, to_delete_user_id).await?;

            if pots_containing_user == 0 {
                return Err(NotFound(format!(
                    "User {} was not part of pot {}",
                    to_delete_user_id, the_pot_id
                )));
            }

            let result = diesel::delete(pots_to_users)
                .filter(user_id.eq(to_delete_user_id))
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(result > 0)
        }

        async fn get_pot_by_id_and_owner(
            &self,
            the_pot_id: i32,
            requester_id: Uuid,
        ) -> Result<Vec<Pot>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            pots.filter(id.eq(the_pot_id).and(owner_id.eq(requester_id)))
                .select(Pot::as_select())
                .load(&mut conn)
                .await
                .map_err(internal_error)
        }

        async fn get_pots_containing_user(
            &self,
            the_pot_id: i32,
            to_delete_user_id: Uuid,
        ) -> Result<i64, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            pots_to_users
                .filter(pot_id.eq(the_pot_id).and(user_id.eq(to_delete_user_id)))
                .count()
                .get_result::<i64>(&mut conn)
                .await
                .map_err(internal_error)
        }

        /// Gets a Vector of all Pots.
        pub async fn get_pots(
            &self,
            user_uuid: Uuid,
        ) -> Result<Vec<(Pot, Vec<User>)>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // getting the pot ids where the user behind user_uuid is only a part of (not the owner)
            let pot_ids: Vec<i32> = pots_to_users
                .filter(user_id.eq(user_uuid))
                .select(pot_id)
                .load(&mut conn)
                .await
                .map_err(internal_error)?;

            // getting all pods where the user behind user_uuid is a part of OR the owner
            let loaded_pots = pots
                .filter(owner_id.eq(user_uuid).or(id.eq_any(pot_ids)))
                .select(Pot::as_select())
                .load(&mut conn)
                .await
                .map_err(not_found_error)?;

            let mut pots_with_users = vec![];

            for pot in loaded_pots {
                let loaded_users = users
                    .left_join(pots_to_users.on(user_id.eq(db_user_id)))
                    .filter(pot_id.eq(pot.id()))
                    .select(User::as_select())
                    .get_results(&mut conn)
                    .await
                    .map_err(internal_error)?;

                pots_with_users.push((pot, loaded_users))
            }

            Ok(pots_with_users)
        }

        /// Gets a Pot with the given id if it exists.
        pub async fn get_pot_by_id(
            &self,
            to_search: i32,
            requester_id: Uuid,
        ) -> Result<Pot, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // getting the pot ids where the user behind user_uuid is only a part of (not the owner)
            let pot_ids: Vec<i32> = pots_to_users
                .filter(user_id.eq(requester_id))
                .select(pot_id)
                .load(&mut conn)
                .await
                .map_err(internal_error)?;

            pots.filter(
                pots_id
                    .eq(to_search)
                    .and(owner_id.eq(requester_id).or(pots_id.eq_any(pot_ids))),
            )
            .get_result::<Pot>(&mut conn)
            .await
            .map_err(not_found_error)
        }

        /// Tries to delete the pot with the given `to_delete` id. Checks if `requester_id` belongs
        /// to the owner of the pot. The pot can only be deleted, if no further expenses are outstanding.
        pub async fn try_delete_pot(
            &self,
            to_delete: i32,
            requester_id: Uuid,
        ) -> Result<bool, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // check if user is even allowed to try to delete the pot
            let is_allowed_to_delete = pots
                .filter(id.eq(to_delete).and(owner_id.eq(requester_id)))
                .count()
                .get_result::<i64>(&mut conn)
                .await
                .map_err(internal_error)?
                == 1;

            if !is_allowed_to_delete {
                return Err(Forbidden(format!(
                    "The user does not own the pot with id {}",
                    to_delete
                )));
            }

            // check if pot has outstanding expenses
            let outstanding = self
                .expense_service
                .get_expenses_by_pot_id(to_delete, requester_id)
                .await;

            if let Err(error) = outstanding {
                return Err(check_error(error));
            }

            for joined_expense in outstanding? {
                for split in joined_expense.1 {
                    if !split.is_paid() {
                        return Err(Conflict(format!(
                            "Cannot delete pot with id {} because there are unpaid expenses",
                            to_delete
                        )));
                    }
                }
            }

            let deleted =
                diesel::delete(pots.filter(id.eq(to_delete).and(owner_id.eq(requester_id))))
                    .execute(&mut conn)
                    .await
                    .map_err(not_found_error)?;

            Ok(deleted == 1)
        }
    }

    /// Creates a new PotService with the given DbConnectionPool.
    pub fn new_service(pool: DbPool) -> PotService {
        PotService {
            db_pool: pool.clone(),
            currency_service: currency_service::new_service(pool.clone()),
            expense_service: expense_service::new_service(pool.clone()),
        }
    }
}
