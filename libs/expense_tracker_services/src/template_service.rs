pub mod pot_template_service {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use diesel::{SelectableHelper, ExpressionMethods, QueryDsl, BoolExpressionMethods};
    use diesel::internal::derives::multiconnection::chrono;
    use diesel::internal::derives::multiconnection::chrono::Datelike;
    use diesel::result::Error;
    use diesel_async::{AsyncConnection, RunQueryDsl};
    use diesel_async::scoped_futures::ScopedFutureExt;
    use log::{debug, error, info, warn};
    use uuid::Uuid;
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::pots::pots::{NewPot, PotToUser};
    use expense_tracker_db::schema::currencies::dsl::currencies;
    use expense_tracker_db::schema::pot_template_users::dsl::pot_template_users;
    use expense_tracker_db::schema::pot_template_users::{pot_template_id, user_id};
    use expense_tracker_db::schema::pot_templates::dsl::{pot_templates, id, owner_id};
    use expense_tracker_db::schema::users::dsl::users;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::template_pots::template_pots::{NewPotTemplate, NewPotTemplateUser, PotTemplate, PotTemplateUser};
    use expense_tracker_db::users::users::User;
    use crate::{internal_error, not_found_error, ExpenseError, CRON_MANAGER_SERVICE};
    use crate::cron_manager_service::cron_manager_service::CronManagerService;
    use crate::currency_service::currency_service;
    use crate::currency_service::currency_service::CurrencyService;
    use crate::ExpenseError::Forbidden;
    use crate::pot_service::pot_service;
    use crate::pot_service::pot_service::PotService;
    use crate::user_service::user_service;
    use crate::user_service::user_service::UserService;

    /// A service offering interfaces related to Templates.
    #[derive(Clone)]
    pub struct PotTemplateService {
        db_pool: DbPool,
        currency_service: CurrencyService,
        user_service: UserService,
        pot_service: PotService,
        cron_manager_service: Arc<Mutex<CronManagerService>>
    }

    impl PotTemplateService {
        /// Creates a new instance of PotTemplateService.
        pub fn new_service(db_pool: DbPool) -> Self {
            Self {
                db_pool: db_pool.clone(),
                currency_service: currency_service::new_service(db_pool.clone()),
                user_service: user_service::new_service(db_pool.clone()),
                pot_service: pot_service::new_service(db_pool.clone()),
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
                        if !new_template_user_ids.iter().any(|u| u.eq(&new_template_clone.owner_id())) {
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
                        let loaded_users = self.user_service
                            .get_users(template_user_uuids)
                            .await?;

                        let currency = self
                            .currency_service
                            .get_currency_by_id(currency_id_clone)
                            .await?;

                        Ok((template_pot, currency, loaded_users))
                    }
                        .scope_boxed()
                })
                .await
                .map_err(not_found_error)?;

            // add cron job for this template and start it
            self.add_template_cron_job(result.0.clone()).await;
            self.start_cron_jobs().await;

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

            let mut cron_manager_service = self.cron_manager_service.lock().await;
            debug!("Got CronManagerService!");
            cron_manager_service.remove_cron_job_with_id(to_delete).await;
            debug!("Removed cron job with id {}", to_delete);

            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let deleted = diesel::delete(
                pot_templates.filter(id.eq(to_delete).and(owner_id.eq(requester_id))))
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(deleted == 1)
        }

        /// Gets the templates owned by the requester.
        pub async fn get_own_templates(&self, requester_id: Uuid)
            -> Result<Vec<(PotTemplate, Currency, Vec<User>)>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            // get templates with their currency
            let templates_with_currency = pot_templates
                .inner_join(currencies)
                .filter(owner_id.eq(requester_id))
                .select((PotTemplate::as_select(), Currency::as_select()))
                .load::<(PotTemplate, Currency)>(&mut conn)
                .await
                .map_err(internal_error)?;

            // Vector that will hold (PotTemplate, Currency, Vec<User>) that will be returned.
            let mut result = vec![];

            for (template, currency) in templates_with_currency{
                let template_id = template.id();

                // join pot_template_users with users
                // filtered by template
                let loaded_users = self
                    .get_users_for_pot_template(&template)
                    .await
                    .map_err(internal_error)?;

                result.push((template, currency, loaded_users));
            }

            Ok(result)
        }

        /// Gets the template with the given id owned by the requester.
        pub async fn get_own_template_by_id(&self,
                                            requester_id: Uuid,
                                            requested_template_id : i32)
                                       -> Result<(PotTemplate, Currency, Vec<User>), ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;


            // // get template with their currency
            let template_with_currency = pot_templates
                .inner_join(currencies)
                .filter(owner_id.eq(requester_id).and(id.eq(requested_template_id)))
                .select((PotTemplate::as_select(), Currency::as_select()))
                .first::<(PotTemplate, Currency)>(&mut conn)
                .await
                .map_err(internal_error)?;


            // join pot_template_users with users
            // filtered by template
            let loaded_users = self
                .get_users_for_pot_template(&template_with_currency.0)
                .await
                .map_err(internal_error)?;

            Ok((template_with_currency.0, template_with_currency.1, loaded_users))
        }

        async fn get_users_for_pot_template(&self, template : &PotTemplate)
            -> Result<Vec<User>, Error> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let template_id = template.id();

            pot_template_users
                .inner_join(users)
                .filter(pot_template_id.eq(template_id))
                .select(User::as_select())
                .load::<User>(&mut conn)
                .await
        }

        /// Used to get all templates from the database.
        async fn get_templates(&self) -> Result<Vec<PotTemplate>, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let result = pot_templates.load::<PotTemplate>(&mut conn)
                .await
                .map_err(internal_error)?;

            Ok(result)
        }

        /// Checks if the given requester_id is the owner of the pot template with the given pot_template_id.
        async fn is_owner(&self, target_pot_template_id: i32, requester_id: Uuid) -> Result<bool, ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;
            let result = pot_templates
                .filter(id.eq(target_pot_template_id).and(owner_id.eq(requester_id)))
                .count()
                .get_result::<i64>(&mut conn)
                .await
                .map_err(internal_error)? == 1;

            Ok(result)
        }

        /// Used to initialize the service, this is called when the service is first created.
        /// This is necessary here so that the CronManagerService can schedule cron jobs after a
        /// reboot.
        pub async fn init_service(&self) {
            debug!("Initializing TemplateService");

            // load the template from the db
            let templates = self.get_templates().await;

            if let Err(e) = templates {
                error!("Could not get templates from database: {}", e);
                return;
            }

            // add cron jobs for each template
            if let Ok(templates) = templates {
                if templates.is_empty() {
                    debug!("No templates found in database, skipping cron job initialization");
                    return;
                }

                for template in templates {
                    // add cron job for this template
                    self.add_template_cron_job(template).await;
                }

                self.start_cron_jobs().await;
            }
        }

        async fn start_cron_jobs(&self) {
            let cron_manager_service_mutex = Arc::clone(&self.cron_manager_service);
            let mut cron_manager_service = cron_manager_service_mutex.lock().await;
            debug!("Acquired cron manager service lock. Running cron jobs...");
            cron_manager_service.run_cron_jobs().await;
            debug!("Cron jobs started!");
        }

        async fn add_template_cron_job(&self, template: PotTemplate) {
            let cron_manager_service_mutex = Arc::clone(&self.cron_manager_service);
            debug!("CronManagerService referenced");

            {
                let mut cron_manager_service = cron_manager_service_mutex.lock().await;
                let cron_expression = template.cron_expression();
                let template_id = template.id();
                let template = template.clone();

                let db_pool_clone = self.db_pool.clone();
                let template_clone = template.clone();
                let pot_service_clone = self.pot_service.clone();

                let function = Box
                ::new(move || {
                    let template_clone = template_clone.clone();
                    let db_pool_clone = db_pool_clone.clone();
                    let pot_service_clone = pot_service_clone.clone();

                    tokio::spawn(async move {
                        Self::cron_job_create_template(
                            &template_clone,
                            &db_pool_clone,
                            &pot_service_clone).await;
                    });
                });

                // add the cron job to the CronManagerService, using the cron expression and the function
                // defined above. If the cron job could not be added, log the error.
                let cron_result = cron_manager_service
                    .add_cron_job_with_id(cron_expression, function, template_id)
                    .await;
                debug!("Added cron job for template with id {}", template_id);

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

        async fn cron_job_create_template(
            template: &PotTemplate,
            db_pool: &DbPool,
            pot_service: &PotService) {
            let mut conn = db_pool.get().await.unwrap();
            // 1. load the users from the database
            let template_id = template.id();
            debug!("Creating new pot from template with id {}", template_id);

            let template_users = pot_template_users
                .filter(pot_template_id.eq(template_id))
                .load::<PotTemplateUser>(&mut conn)
                .await;

            debug!("Loaded users for template {}", template_id);

            if let Err(e) = template_users {
                error!("Could not load users for template {}: {}", template_id, e);
                return;
            }

            // unwrap should be safe here, since we checked if the query was successful in the previous step!
            let template_user_list = template_users.unwrap();

            // 2. fill in template placeholders: {month} {year} - leave rest unchanged,
            // e.g. Home {month}.{year} should be turned into: Home 05.2026
            let template_name = Self::replace_placeholders(template.name());
            debug!("Template name: {}", template_name);

            // 3. create a new pot automatically using the information from the pot template
            let new_pot = NewPot::from_template(template, &template_name);

            let create_pot_result = pot_service.create_pot(new_pot).await;
            if let Err(error) = create_pot_result {
                error!("Failed to create a new pot from template {}: {}", template_id, error);
                return;
            }

            // pot creation must have been successful then!
            let pot = create_pot_result.unwrap().0;
            let pot_id = pot.id();
            debug!("Created new pot from template {}: {}", template_name, pot_id);
            let pot_owner_id = pot.owner_id();
            let mut pots_to_users = vec![];

            for user in template_user_list {
                debug!("Adding user {} to pot {}", user.user_id(), pot_id);
                pots_to_users.push(PotToUser::new(pot_id, user.user_id()));
            }

            let users_to_add = pots_to_users.len();

            // add users
            let add_users_result = pot_service
                .add_users_to_pot(pots_to_users, pot_owner_id)
                .await;

            if let Err(error) = add_users_result {
                error!("Could not add users to pot {} on template creation: {}", pot_id, error);
                return;
            }

            // no error, now check if any users have been added
            if let Ok(users_added) = add_users_result
            {
                if !users_added && users_to_add > 0 {
                    warn!(
                        "No users added to pot {} on template creation, should have been {}.\
                        \nReview database template {}!",
                        pot_id,
                        users_to_add,
                        template_id);
                    return;
                }

                info!("Added users to pot {} on template creation", pot_id);
            }
        }

        /// Replacing the placeholders inside a string with the current month and year.
        /// The placeholders are {month} and {year}.
        /// Example: "Home {month}.{year}" -> "Home 05.2026"
        /// Example: "Home {month}" -> "Home 05"
        /// Example: "Home" -> "Home"
        /// Example: "{month}" -> "{month}"
        /// Example: "{month}.{year}" -> "{month}.{year}"
        /// Example: "{month} {year}" -> "{month} {year}"
        /// Example: "{month} {year} {month}" -> "{month} {year} {month}"
        /// Example: "{month} {year} {month} {year}" -> "{month} {year} {month} {year}"
        pub(crate) fn replace_placeholders(template_name: &str) -> String {
            let mut new_template_name = template_name.to_string();
            new_template_name = new_template_name.replace("{month}", &format!("{:02}", chrono::Local::now().month()));
            new_template_name = new_template_name.replace("{year}", &format!("{}", chrono::Local::now().year()));
            debug!("Replaced placeholders in template name: {} -> {}", template_name, new_template_name);
            new_template_name
        }
    }
}

#[cfg(test)]
mod test {
    use diesel::internal::derives::multiconnection::chrono;
    use diesel::internal::derives::multiconnection::chrono::Datelike;
    use crate::template_service::pot_template_service::PotTemplateService;

    // the built-in test framework does not support parametrized tests yet, so we either have
    // to use a macro or define multiple tests for each parameter
    #[test]
    fn test_replace_placeholders() {
        let now = chrono::Local::now();

        let template_name = "Home {month}.{year}";
        let expected = format!("Home {:02}.{}", now.month(), now.year());
        let result = PotTemplateService::replace_placeholders(template_name);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_placeholders_no_placeholders() {
        let template_name = "Home";
        let expected = "Home";
        let result = PotTemplateService::replace_placeholders(template_name);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_placeholders_fake_placeholders() {
        let template_name = "Home {placeholder} {does} {not} {exist}";
        let expected = "Home {placeholder} {does} {not} {exist}";
        let result = PotTemplateService::replace_placeholders(template_name);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_placeholders_empty_placeholders() {
        let template_name = "Home {}";
        let expected = "Home {}";
        let result = PotTemplateService::replace_placeholders(template_name);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_placeholders_american_format() {
        let now = chrono::Local::now();
        let template_name = "Home {year}/{month}";
        let expected = format!("Home {}/{:02}", now.year(), now.month());
        let result = PotTemplateService::replace_placeholders(template_name);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_placeholders_crazy_format() {
        let now = chrono::Local::now();
        let template_name = "{year}Ho{month}me";
        let expected = format!("{}Ho{:02}me", now.year(), now.month());
        let result = PotTemplateService::replace_placeholders(template_name);
        assert_eq!(result, expected);
    }
}