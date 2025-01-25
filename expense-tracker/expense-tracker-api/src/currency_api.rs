pub mod currency_api {
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::Json;
    use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, ExpressionMethods};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use expense_tracker_db::currencies::currencies::{Currency, NewCurrency};
    use expense_tracker_db::schema::currencies::dsl::currencies;
    use expense_tracker_db::schema::currencies::symbol;
    use expense_tracker_db::setup::{DbConnectionPool, DbPool};
    use crate::api::internal_error;

    /// Registers all functions of the Currency API.
    pub fn register(pool : DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(create_currency))
            .routes(routes!(get_currencies))
            .with_state(pool)
    }

    /// DTO representing a currency.
    #[derive(ToSchema, Serialize)]
    pub struct CurrencyDTO {
        id: i32,
        name: String,
        symbol: String
    }

    impl Clone for CurrencyDTO {
        fn clone(&self) -> Self {
            Self {
                id: self.id,
                name: self.name.clone(),
                symbol: self.symbol.clone()
            }
        }
    }

    impl CurrencyDTO {
        /// Converts Currency to CurrencyDTO.
        pub fn from(src : Currency) -> Self {
            Self {
                id: src.id(),
                name: src.name().to_string(),
                symbol: src.symbol().to_string()
            }
        }

        /// Converts a Vec<Currency> to Vec<CurrencyDTO>.
        pub fn from_vec(src : Vec<Currency>) -> Vec<Self> {
            let mut dest = Vec::new();

            for currency in src {
                dest.push(CurrencyDTO::from(currency));
            }

            dest
        }

        pub fn id(&self) -> i32 {
            self.id
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn symbol(&self) -> &str {
            &self.symbol
        }
    }

    /// DTO representing a new currency.
    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewCurrencyDTO {
        name: String,
        symbol: String
    }

    impl NewCurrencyDTO {
        /// Converts the DTO to the respective db model.
        pub fn to_db(&self) -> NewCurrency {
            NewCurrency::new(self.name.clone(), self.symbol.clone())
        }
    }

    /// Creates a new currency, if symbol is not yet in use.
    #[utoipa::path(
        post,
        path = "/currencies",
        tag  = "Currency",
        responses(
            (status = 200, description = "The currency has been created", body = NewCurrencyDTO),
            (status = 409, description = "Detected a conflict, as the symbol is already known.")
        ),
        request_body = NewCurrencyDTO
    )]
    pub async fn create_currency(
        State(pool) : State<DbPool>,
        Json(new_currency) : Json<NewCurrencyDTO>
    ) -> Result<Json<CurrencyDTO>, (StatusCode, String)> {
        let conn = pool.get().await.map_err(internal_error)?;

        let currency_symbol = new_currency.symbol.clone();

        let res = conn
            .interact(|conn| currencies
                .filter(symbol.eq(currency_symbol))
                .first::<Currency>(conn)
            )
            .await.map_err(internal_error)?;

        // ok means, that we found a currency for the given symbol!
        if res.is_ok() {
            let currency_symbol = new_currency.symbol.clone();
            return Err(
                (
                    StatusCode::CONFLICT,
                    format!("There is already a currency with symbol {}", currency_symbol)
                )
            );
        }

        let res = conn
            .interact(move |conn| {
                diesel::insert_into(currencies)
                    .values(new_currency.to_db())
                    .returning(Currency::as_returning())
                    .get_result::<Currency>(conn)
            })
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        Ok(Json(CurrencyDTO::from(res)))
    }

    /// Gets a list of all currently known currencies.
    #[utoipa::path(
        get,
        path = "/currencies",
        tag = "Currency",
        responses(
            (status = 200, description = "All currencies known to the system", body = Vec<CurrencyDTO>),
        )
    )]
    pub async fn get_currencies(
        State(pool): State<DbPool>
    ) -> Result<Json<Vec<CurrencyDTO>>, (StatusCode, String)> {
        let conn  = pool.get().await.map_err(internal_error)?;

        let loaded_currencies = conn
            .interact(|conn| currencies
                .select(Currency::as_select())
                .load::<Currency>(conn)
            )
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        Ok(Json(CurrencyDTO::from_vec(loaded_currencies)))
    }
}