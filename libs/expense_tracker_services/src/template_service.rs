pub mod pot_template_service {
    use diesel::{SelectableHelper, ExpressionMethods, QueryDsl, BoolExpressionMethods};
    use diesel::result::Error;
    use diesel_async::{AsyncConnection, RunQueryDsl};
    use diesel_async::scoped_futures::ScopedFutureExt;
    use expense_tracker_db::schema::pot_template_users::dsl::pot_template_users;
    use expense_tracker_db::schema::pot_templates::dsl::{pot_templates, id, owner_id};
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::template_pots::template_pots::{NewPotTemplate, NewPotTemplateUser, PotTemplate, PotTemplateUser};
    use crate::{internal_error, not_found_error, ExpenseError};
    use crate::ExpenseError::Forbidden;

    /// A service offering interfaces related to Templates.
    #[derive(Clone)]
    pub struct PotTemplateService {
        db_pool: DbPool
    }

    impl PotTemplateService {
        /// Creates a new template with associated users.
        pub async fn create_template(
            &self,
            new_template: NewPotTemplate,
            new_template_users: Vec<NewPotTemplateUser>
        ) -> Result<(PotTemplate, Vec<PotTemplateUser>), ExpenseError> {
            let mut conn = self.db_pool.get().await.map_err(internal_error)?;

            let result = conn
                .transaction::<_, Error, _>(|conn| {
                    async move {
                        let template_pot = diesel::insert_into(pot_templates)
                            .values(new_template)
                            .returning(PotTemplate::as_returning())
                            .get_result::<PotTemplate>(conn)
                            .await?;

                        let template_users = diesel::insert_into(pot_template_users)
                            .values(new_template_users)
                            .returning(PotTemplateUser::as_returning())
                            .get_results::<PotTemplateUser>(conn)
                            .await?;

                        Ok((template_pot, template_users))
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