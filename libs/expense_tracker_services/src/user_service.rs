pub mod user_service {
    use crate::{internal_error, ExpenseError};
    use expense_tracker_db::users::users::User;
    use uuid::Uuid;
    use expense_tracker_db::user_repository::UserRepository;

    /// A service to interact with user context.
    #[derive(Clone)]
    pub struct UserService<R: UserRepository> {
        user_repository: R
    }

    impl <R: UserRepository> UserService<R> {
        /// Creates a new UserService.
        pub fn new(user_repository : R) -> Self {
            Self { user_repository }
        }

        /// Creates a new user given the new_user data.
        pub async fn create_user(&self, new_user: User) -> Result<User, ExpenseError> {
            self.user_repository.create_user(new_user).await.map_err(internal_error)
        }

        /// Gets all users in the database.
        pub async fn get_users(&self) -> Result<Vec<User>, ExpenseError> {
            self.user_repository.get_users().await.map_err(internal_error)
        }

        /// Gets the user by the given Uuid. Returns a NotFoundError if no user with the given Id
        /// exists.
        pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, ExpenseError> {
            self.user_repository.get_user_by_id(user_id).await.map_err(internal_error)
        }
    }
}

#[cfg(test)]
mod test {
    use diesel::result::DatabaseErrorKind;
    use mockall::predicate;
    use uuid::Uuid;
    use expense_tracker_db::user_repository::MockUserRepository;
    use expense_tracker_db::users::users::User;
    use crate::ExpenseError;
    use crate::user_service::user_service::UserService;

    #[test]
    fn new_service_can_be_constructed() {
        let mocked_user_repository = MockUserRepository::default();
        let service = UserService::new(mocked_user_repository);
    }

    #[tokio::test]
    async fn create_user_returns_ok() {
        // Arrange
        let user = User::new(Uuid::new_v4(), "John Doe".to_string());
        let expected_user = user.clone();
        let user_clone = user.clone();

        let mut mocked_user_repository = MockUserRepository::new();
        mocked_user_repository
            .expect_create_user()
            .with(predicate::eq(user))
            .returning(|user| Ok(user))
            .once();

        let service = UserService::new(mocked_user_repository);

        // Act
        let result = service.create_user(user_clone).await;

        // Assert
        assert_eq!(true, result.is_ok());

        if let Ok(result) = result{
            assert_eq!(expected_user, result);
        }
    }

    #[tokio::test]
    async fn create_user_returns_error() {
        // Arrange
        let user = User::new(Uuid::new_v4(), "John Doe".to_string());
        let user_clone = user.clone();

        let mut mocked_user_repository = MockUserRepository::new();
        mocked_user_repository
            .expect_create_user()
            .with(predicate::eq(user))
            .returning(|user| Err(
                diesel::result::Error::DatabaseError(
                    DatabaseErrorKind::UnableToSendCommand,
                    Box::new("Unable to send command".to_string())
                )))
            .once();

        let service = UserService::new(mocked_user_repository);

        // Act
        let result = service.create_user(user_clone).await;

        // Assert
        assert_eq!(true, result.is_err());

        if let Err(result) = result{
            assert_eq!(ExpenseError::Internal("Unable to send command".to_string()), result);
        }
    }
}