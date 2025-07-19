pub mod currency_api {
    use crate::api::{check_error, ApiResponse};
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::Json;
    use expense_tracker_db::currencies::currencies::{Currency, NewCurrency};
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_services::currency_service::currency_service;
    use expense_tracker_services::currency_service::currency_service::CurrencyService;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;

    /// Registers all functions of the Currency API.
    pub fn register(pool: DbPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(create_currency))
            .routes(routes!(get_currencies))
            .with_state(currency_service::new_service(pool))
    }

    /// DTO representing a currency.
    #[derive(ToSchema, Serialize)]
    pub struct CurrencyDTO {
        id: i32,
        name: String,
        symbol: String,
    }

    impl Clone for CurrencyDTO {
        fn clone(&self) -> Self {
            Self {
                id: self.id,
                name: self.name.clone(),
                symbol: self.symbol.clone(),
            }
        }
    }

    impl CurrencyDTO {
        /// Converts Currency to CurrencyDTO.
        pub fn from(src: Currency) -> Self {
            Self {
                id: src.id(),
                name: src.name().to_string(),
                symbol: src.symbol().to_string(),
            }
        }

        /// Converts a Vec of Currency to Vec of CurrencyDTO.
        pub fn from_vec(src: Vec<Currency>) -> Vec<Self> {
            let mut dest = Vec::new();

            for currency in src {
                dest.push(CurrencyDTO::from(currency));
            }

            dest
        }

        pub fn id(&self) -> i32 {
            self.id
        }
    }

    /// DTO representing a new currency.
    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewCurrencyDTO {
        name: String,
        symbol: String,
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
            (status = 201, description = "The currency has been created", body = NewCurrencyDTO),
            (status = 409, description = "Detected a conflict, as the symbol is already known.")
        ),
        request_body = NewCurrencyDTO,
        security(
                ("bearer" = [])
        )
    )]
    pub async fn create_currency(
        State(service): State<CurrencyService>,
        Json(new_currency): Json<NewCurrencyDTO>,
    ) -> Result<ApiResponse<CurrencyDTO>, ApiResponse<String>> {
        let res = service
            .create_currency(new_currency.to_db())
            .await
            .map_err(check_error)?;

        Ok((StatusCode::CREATED, Json(CurrencyDTO::from(res))))
    }

    /// Gets a list of all currently known currencies.
    #[utoipa::path(
        get,
        path = "/currencies",
        tag = "Currency",
        responses(
            (status = 200, description = "All currencies known to the system", body = Vec<CurrencyDTO>),
        ),
        security(
                ("bearer" = [])
        )
    )]
    pub async fn get_currencies(
        State(service): State<CurrencyService>,
    ) -> Result<ApiResponse<Vec<CurrencyDTO>>, ApiResponse<String>> {
        let loaded_currencies = service.get_currencies().await.map_err(check_error)?;

        Ok((
            StatusCode::OK,
            Json(CurrencyDTO::from_vec(loaded_currencies)),
        ))
    }
}
