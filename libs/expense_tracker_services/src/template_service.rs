pub mod pot_template_service {
    use std::sync::{Arc, Mutex};
    use diesel::{SelectableHelper, ExpressionMethods, QueryDsl, BoolExpressionMethods};
    use diesel::result::Error;
    use diesel_async::{AsyncConnection, RunQueryDsl};
    use diesel_async::scoped_futures::ScopedFutureExt;
    use log::{debug, error};
    use uuid::Uuid;
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::schema::pot_template_users::dsl::pot_template_users;
    use expense_tracker_db::schema::pot_template_users::{pot_template_id, user_id};
    use expense_tracker_db::schema::pot_templates::dsl::{pot_templates, id, owner_id};
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::template_pots::template_pots::{NewPotTemplate, NewPotTemplateUser, PotTemplate, PotTemplateUser};
    use expense_tracker_db::users::users::User;
    use crate::{check_error, internal_error, not_found_error, ExpenseError, CRON_MANAGER_SERVICE};
    use crate::cron_manager_service::cron_manager_service::CronManagerService;
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
        cron_manager_service: Arc<Mutex<CronManagerService>>
    }

    impl PotTemplateService {
        /// Creates a new instance of PotTemplateService.
        pub fn new_service(db_pool: DbPool) -> Self {
             Self {
                db_pool: db_pool.clone(),
                currency_service: currency_service::new_service(db_pool.clone()),
                user_service: user_service::new_service(db_pool.clone()),
                // we need to clone the Arc because we want to be able to use the Arc in the background service
                cron_manager_service: Arc::clone(&CRON_MANAGER_SERVICE)
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
                        for user_uuid in new_template_user_ids {
                            template_users.push(NewPotTemplateUser::new(user_uuid, template_pot.clone().id()));
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

        /// Adds the given users to the given pot template, if they exist and aren't already part of the template.
        pub async fn add_users_to(&self, target_pot_template_id: i32, users_to_add: Vec<Uuid>, requester_id: Uuid)
                                  -> Result<bool, ExpenseError> {
            // if not requested by the owner, stop at once - the frontend should not allow this
            if !self.is_owner(target_pot_template_id, requester_id).await? {
                return Err(Forbidden(format!(
                    "The user does not own the pot template with id {}",
                    target_pot_template_id
                )));
            }

            let mut new_pot_template_users = vec![];

            for user_uuid in users_to_add {
                new_pot_template_users.push(NewPotTemplateUser::new(user_uuid, target_pot_template_id))
            }

            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            diesel::insert_into(pot_template_users)
                .values(&new_pot_template_users)
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;
            Ok(true)
        }

        /// Removes the given users from the given pot template, if they are part of the template.
        pub async fn remove_users_from(&self, target_pot_template_id: i32, users_to_remove: Vec<Uuid>, requester_id: Uuid)
                                       -> Result<bool, ExpenseError> {
            // if not requested by the owner, stop at once - the frontend should not allow this
            if !self.is_owner(target_pot_template_id, requester_id).await? {
                return Err(Forbidden(format!(
                    "The user does not own the pot template with id {}",
                    target_pot_template_id
                )));
            }

            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            diesel::delete(pot_template_users
                .filter(pot_template_id.eq(target_pot_template_id).and(user_id.eq_any(users_to_remove))))
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(true)
        }

        /// Deletes the template with the given id, only if the user with the given id is the owner of the template.
        pub async fn delete_template(
            &self,
            to_delete: i32,
            requester_id: Uuid) -> Result<bool, ExpenseError> {
            // if not requested by the owner, stop at once - the frontend should not allow this
            if !self.is_owner(to_delete, requester_id).await? {
                return Err(Forbidden(format!(
                    "The user does not own the pot template with id {}",
                    to_delete
                )));
            }

            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let deleted = diesel::delete(
                pot_templates.filter(id.eq(to_delete).and(owner_id.eq(requester_id))))  
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(deleted == 1)
        }

        /// Used to get all templates from the database.
        pub async fn get_templates(&self) -> Result<Vec<PotTemplate>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let result = pot_templates.load::<PotTemplate>(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(result)
        }

        /// Checks if the given requester_id is the owner of the pot template with the given pot_template_id.
        async fn is_owner(&self, target_pot_template_id : i32, requester_id : Uuid) -> Result<bool, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let result = pot_templates.filter(id.eq(target_pot_template_id).and(owner_id.eq(requester_id)))
                .count()
                .get_result::<i64>(&mut conn)
                .await
                .map_err(internal_error)? == 1;

            Ok(result)
        }

        /// Used to initialize the service, this is called when the service is first created.
        /// This is necessary here so that the CronManagerService can schedule cron jobs after a
        /// reboot.
        async fn init_service(&self) {
            debug!("Initializing TemplateService");

            // load the template from the db
            let templates = self.get_templates().await;

            if let Err(e) = templates {
                error!("Could not get templates from database: {}", e);
                return;
            }

            // add cron jobs for each template
            if let Ok(templates) = templates {
                let cron_manager_service_mutex = Arc::clone(&self.cron_manager_service);
                debug!("CronManagerService referenced");

                for template in templates {
                    if let Ok(mut cron_manager_service) = cron_manager_service_mutex.lock() {
                        let cron_expression = template.cron_expression();
                        let template_id = template.id();
                        let template = template.clone();

                        let db_pool_clone = self.db_pool.clone();
                        let template_clone = template.clone();
                        let function = Box
                        ::new(move || {
                            Self::cron_job_create_template(&template_clone, &db_pool_clone)
                        });

                        // add the cron job to the CronManagerService, using the cron expression and the function
                        // defined above. If the cron job could not be added, log the error.
                        let cron_result = cron_manager_service
                            .add_cron_job_with_id(cron_expression, function, template_id)
                            .await;

                        if let Err(ref cron_error) = cron_result {
                            error!("Could not add cron job for template {}: {}", template_id, cron_error);
                        }

                        if let Ok(cron_id) = cron_result {
                            debug!(
                                "Added cron job for template with id {} and cron expression {}",
                                cron_id,
                                cron_expression);
                        }
                    }

                }
            }
        }

       fn cron_job_create_template(template: &PotTemplate, db_pool: &DbPool) {
            // TODO:
            // 1. load the template from the db
            // 2. fill in template placeholders: {month} {year} - leave rest unchanged, e.g. Home {month}.{year} should be turned into : Home 05.2026
            // 3. load the users for the template
            // 4. create a new pot automatically using the information from the pot template
        }
    }
}