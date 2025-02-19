pub mod user_service {
    use diesel::{RunQueryDsl, SelectableHelper};
    use expense_tracker_db::setup::DbConnectionPool;
    use expense_tracker_db::schema as expense_tracker_db_schema;
    use expense_tracker_db::users::users::{NewUser, User};
    use crate::{internal_error, not_found_error, ExpenseError};

    /// A service to interact with user context.
    #[derive(Clone)]
    pub struct UserService {
        db_pool : DbConnectionPool
    }

    impl UserService {
        /// Creates a new user given the new_user data.
        pub async fn create_user(&self, new_user: NewUser) -> Result<User, ExpenseError> {
            let conn = self.db_pool.get().await.map_err(internal_error)?;

            let res = conn
                .interact(move |conn| {
                    diesel::insert_into(expense_tracker_db_schema::users::table)
                        .values(new_user)
                        .returning(User::as_returning())
                        .get_result::<User>(conn)
                })
                .await
                .map_err(internal_error)?
                .map_err(not_found_error)?;

            Ok(res)
        }
    }

    /// Creates a new UserService.
    pub fn create_service(pool : DbConnectionPool) -> UserService {
        UserService {
            db_pool : pool
        }
    }
}