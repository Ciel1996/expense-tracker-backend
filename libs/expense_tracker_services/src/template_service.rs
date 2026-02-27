pub mod pot_template_service {
    use diesel::{SelectableHelper, ExpressionMethods, QueryDsl, BoolExpressionMethods};
    use diesel::result::Error;
    use diesel_async::{AsyncConnection, RunQueryDsl};
    use diesel_async::scoped_futures::ScopedFutureExt;
    use uuid::Uuid;
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::schema::pot_template_users::dsl::pot_template_users;
    use expense_tracker_db::schema::pot_templates::dsl::{pot_templates, id, owner_id};
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::template_pots::template_pots::{NewPotTemplate, NewPotTemplateUser, PotTemplate, PotTemplateUser};
    use expense_tracker_db::users::users::User;
    use crate::{check_error, internal_error, not_found_error, ExpenseError};
    use crate::currency_service::currency_service;
    use crate::currency_service::currency_service::CurrencyService;
    use crate::ExpenseError::Forbidden;
    use crate::user_service::user_service;
    use crate::user_service::user_service::UserService;

    /// A service offering interfaces related to Templates.
    #[derive(Clone)]
    pub struct PotTemplateService {
        db_pool: DbPool,
        currency_service: CurrencyService,
        user_service: UserService,
    }

    impl PotTemplateService {
        /// Creates a new instance of PotTemplateService.
        pub fn new_service(db_pool: DbPool) -> Self {
            Self {
                db_pool: db_pool.clone(),
                currency_service: currency_service::new_service(db_pool.clone()),
                user_service: user_service::new_service(db_pool.clone()),
            }
        }

        /// Creates a new template with associated users.
        /// The owner will be added to the list of users automatically inside the function.
        pub async fn create_template(
            &self,
            new_template: NewPotTemplate,
            mut new_template_user_ids: Vec<Uuid>
        ) -> Result<(PotTemplate, Currency, Vec<User>), ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // Using a transaction to ensure that the template is created together with the users in the template users table
            let result = conn
                .transaction::<_, Error, _>(|conn| {
                    async move {
                        // cloning variables for later use and before they are moved
                        let currency_id_clone = new_template.default_currency_id().clone();
                        let new_template_clone = new_template.clone();

                        let template_pot = diesel::insert_into(pot_templates)
                            .values(new_template)
                            .returning(PotTemplate::as_returning())
                            .get_result::<PotTemplate>(conn)
                            .await?;

                        // although the code documentation states that the owner is always the first user in the template users list,
                        // we are checking if the owner is already in the list of users, if not, we add him to the list of users
                        if !new_template_user_ids.iter().any(|u| u.eq(&new_template_clone.owner_id())){
                            new_template_user_ids.push(new_template_clone.owner_id());
                        }

                        // converting the vector of uuids to a vector of NewPotTemplateUser, which diesel
                        // can understand and insert into the database with a reference to the PotTemplates
                        let mut template_users = vec![];
                        for user_id in new_template_user_ids {
                            template_users.push(NewPotTemplateUser::new(user_id, template_pot.clone().id()));
                        }

                        let template_users = diesel::insert_into(pot_template_users)
                            .values(template_users)
                            .returning(PotTemplateUser::as_returning())
                            .get_results::<PotTemplateUser>(conn)
                            .await?;

                        // since user_service.get_users expects an Option<Vec<Uuid>>, we wrap the result
                        // in a Some(Vec<Uuid>)
                        let template_user_uuids = Some(
                            template_users
                            .iter()
                            .map(|u| u.user_id())
                            .collect()
                        );

                        // get the users from the general users table,
                        // because a user can only be added to a template if he is already a user in general!
                        let users = self.user_service
                            .get_users(template_user_uuids)
                            .await?;

                        let currency = self
                            .currency_service
                            .get_currency_by_id(currency_id_clone)
                            .await
                            .map_err(check_error)?;

                        Ok((template_pot, currency, users))
                    }
                    .scope_boxed()
                })
                .await
                .map_err(not_found_error)?;

            Ok(result)
        }

        /// Deletes the template with the given id, only if the user with the given id is the owner of the template.
        pub async fn delete_template(
            &self,
            to_delete: i32,
            requester_id: uuid::Uuid) -> Result<bool, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // check if the user is even allowed to try to delete the pot
            let is_allowed_to_delete = pot_templates
                .filter(id.eq(to_delete).and(owner_id.eq(requester_id)))
                .count()
                .get_result::<i64>(&mut conn)
                .await
                .map_err(internal_error)?
                == 1;

            if !is_allowed_to_delete {
                return Err(Forbidden(format!(
                    "The user does not own the pot template with id {}",
                    to_delete
                )));
            }

            let deleted = diesel::delete(
                pot_templates.filter(id.eq(to_delete).and(owner_id.eq(requester_id))))  
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(deleted == 1)
        }
    }
}