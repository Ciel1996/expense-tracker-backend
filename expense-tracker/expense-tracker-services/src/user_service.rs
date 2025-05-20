pub mod user_service {
    use diesel::{QueryDsl, ExpressionMethods};
    use diesel_async::RunQueryDsl;
    use uuid::{Uuid};
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::schema as expense_tracker_db_schema;
    use expense_tracker_db::schema::users::id;
    use expense_tracker_db::users::users::User;
    use crate::{internal_error, not_found_error, ExpenseError};

    /// A service to interact with user context.
    #[derive(Clone)]
    pub struct UserService {
        db_pool : DbPool
    }

    impl UserService {
        /// Creates a new user given the new_user data.
        pub async fn create_user(&self, new_user: User) -> Result<User, ExpenseError> {
            let mut conn =
                self.db_pool.get().await.map_err(internal_error)?;

            let res = diesel::insert_into(expense_tracker_db_schema::users::table)
                .values(new_user)
                .get_result(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(res)
        }

        /// Gets all users in the database.
        pub async fn get_users(&self) -> Result<Vec<User>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            let users = expense_tracker_db_schema::users::table
                .get_results(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(users)
        }

        /// Gets the user by the given Uuid. Returns a NotFoundError if no user with the given Id
        /// exists.
        pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            let user = expense_tracker_db_schema::users::table
                .filter(id.eq(&user_id))
                .first::<User>(&mut conn)
                .await
                .map_err(not_found_error)?;

            Ok(user)
        }
    }

    /// Creates a new UserService.
    pub fn create_service(pool : DbPool) -> UserService {
        UserService {
            db_pool : pool
        }
    }
}