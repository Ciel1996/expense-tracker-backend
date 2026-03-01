use diesel::{ExpressionMethods, QueryDsl, QueryResult};
use diesel_async::RunQueryDsl;
use mockall::automock;
use uuid::Uuid;
use crate::schema::users::id;
use crate::setup::DbPool;
use crate::users::users::User;
use crate::schema as expense_tracker_db_schema;

/// A trait defining what a user repository should offer.
#[automock]
pub trait UserRepository {
    /// Used to create a new user.
    async fn create_user(&self, user: User) -> QueryResult<User>;
    /// Used to get all users.
    async fn get_users(&self) -> QueryResult<Vec<User>>;
    /// Used to get a user by id.
    async fn get_user_by_id(&self, user_id: Uuid) -> QueryResult<User>;
}

pub struct UserRepositoryPostgres {
    db_pool: diesel_async::pooled_connection::bb8::Pool<diesel_async::AsyncPgConnection>,
}

impl UserRepositoryPostgres {
    fn new(db_pool: DbPool) -> impl UserRepository {
        Self { db_pool }
    }
}
impl UserRepository for UserRepositoryPostgres {
    async fn create_user(&self, user: User) -> QueryResult<User> {
        let mut conn = self.db_pool.get().await.unwrap();

        diesel::insert_into(expense_tracker_db_schema::users::table)
                .values(user)
                .get_result::<User>(&mut conn)
                .await
    }

    async fn get_users(&self) -> QueryResult<Vec<User>>{
        let mut conn = self.db_pool.get().await.unwrap();

        expense_tracker_db_schema::users::table
                .load::<User>(&mut conn)
                .await
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> QueryResult<User> {
        let mut conn = self.db_pool.get().await.unwrap();

        expense_tracker_db_schema::users::table
                    .filter(id.eq(&user_id))
                    .first::<User>(&mut conn)
                    .await
    }
}
