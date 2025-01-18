pub mod pot_api {
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::Json;
    use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl, SelectableHelper};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::pots::pots::{NewPot, Pot};
    use expense_tracker_db::schema as expense_tracker_db_schema;
    use expense_tracker_db::schema::currencies::dsl::currencies;
    use expense_tracker_db::schema::currencies::id;
    use expense_tracker_db::schema::pots::dsl::pots;
    use expense_tracker_db::setup::{DbConnectionPool, DbPool};
    use crate::api::internal_error;
    use crate::currency_api;
    use crate::currency_api::currency_api::CurrencyDTO;

    /// Registers all functions of the Pot API.
    pub fn register(pool : DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(create_pot))
            .routes(routes!(get_pots))
            .with_state(pool)
    }

    /// DTO used when working with existing Pots.
    #[derive(ToSchema, Serialize)]
    pub struct PotDTO {
        id: i32,
        owner_id: i32,
        name: String,
        default_currency: CurrencyDTO,
    }

    impl PotDTO {
        /// Creates a new PotDTO from a db Pot.
        pub fn from(pot : Pot, default_currency : CurrencyDTO) -> Self {
            PotDTO {
                id: pot.id(),
                owner_id: pot.owner_id(),
                name: pot.name().to_string(),
                default_currency
            }
        }

        /// Create a vec<PotDTO> from a vec<Pot>.
        pub fn from_vec(pot_vec : Vec<Pot>, currency_vec : Vec<CurrencyDTO>) -> Vec<Self> {
            let mut dtos : Vec<PotDTO> = vec![];

            for pot in pot_vec {
                let pot_currency = currency_vec
                    .iter()
                    .find(|c| c.id() == pot.default_currency_id());

                if let Some(pot_currency) = pot_currency {
                    dtos.push(PotDTO::from(pot, (*pot_currency).clone()))
                }
            }

            dtos
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
            (status = 201, description = "The pot has been created", body = PotDTO),
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

        let loaded_pot_currency_id = res.default_currency_id().clone();

        // TODO: will most likely need some kind of service layer for stuff like this!
        let currency = conn
            .interact(move |conn| currencies
                .filter(id.eq(loaded_pot_currency_id))
                .first::<Currency>(conn)
            )
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        Ok(Json(PotDTO::from(res, CurrencyDTO::from(currency))))
    }

    /// Gets the list of all pots.
    #[utoipa::path(
        get,
        path = "/pots",
        tag = "Pots",
        responses(
            (status = 200, description = "The list of known pots.", body = Vec<PotDTO>)
        )
    )]
    pub async fn get_pots(
        State(pool): State<DbPool>
    ) -> Result<Json<Vec<PotDTO>>, (StatusCode, String)> {
        let mut conn = pool.get().await.map_err(internal_error)?;

        let loaded_pots = conn
            .interact(|conn| pots
                    .select(Pot::as_select())
                    .load::<Pot>(conn)
            )
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        // TODO: this must be replace by call to a service layer.
        let loaded_currencies =
            currency_api::currency_api::get_currencies(State(pool)).await?.0;

        Ok(Json(PotDTO::from_vec(loaded_pots, loaded_currencies)))
    }
}