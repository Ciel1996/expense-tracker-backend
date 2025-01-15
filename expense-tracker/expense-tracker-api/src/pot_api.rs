pub mod pot_api {
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::Json;
    use diesel::{RunQueryDsl, SelectableHelper};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use expense_tracker_db::pots::pots::{NewPot, Pot};
    use expense_tracker_db::schema as expense_tracker_db_schema;
    use expense_tracker_db::setup::{DbConnectionPool, DbPool};
    use expense_tracker_db::users::users::User;
    use crate::api::internal_error;

    /// Registers all functions of the Pot API.
    pub fn register(pool : DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(create_pot))
            .with_state(pool)
    }

    /// DTO used when working with existing Pots.
    #[derive(ToSchema, Serialize)]
    pub struct PotDTO {
        id: i32,
        owner_id: i32,
        name: String,
        default_currency_id: i32,
    }

    impl PotDTO {
        /// Creates a new PotDTO from a db Pot.
        pub fn from(pot : Pot) -> Self {
            PotDTO {
                id: pot.id(),
                owner_id: pot.owner_id(),
                name: pot.name().to_string(),
                default_currency_id: pot.default_currency_id()
            }
        }
    }

    /// DTO used when creating a new Pot.
    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewPotDTO {
        owner_id: i32,
        name: String,
        default_currency_id: i32,
    }

    impl NewPotDTO {
        /// Converts the DTO to the db object.
        fn to_db(&self) -> NewPot {
            NewPot::new(
                self.owner_id,
                self.name.clone(),
                self.default_currency_id
            )
        }
    }

    /// Creates a pot from the given DTO.
    #[utoipa::path(
        post,
        path = "/pots",
        tag = "Pots",
        responses(
            (status = 201, description = "The pot has been created"),
        ),
        request_body = NewPotDTO
    )]
    pub async fn create_pot(
        State(pool) : State<DbPool>,
        Json(new_pot) : Json<NewPotDTO>
    ) -> Result<Json<PotDTO>, (StatusCode, String)> {
        let conn = pool.get().await.map_err(internal_error)?;

        let res = conn
            .interact(move |conn| {
                diesel::insert_into(expense_tracker_db_schema::pots::table)
                    .values(new_pot.to_db())
                    .returning(Pot::as_returning())
                    .get_result::<Pot>(conn)
            })
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        Ok(Json(PotDTO::from(res)))
    }
}