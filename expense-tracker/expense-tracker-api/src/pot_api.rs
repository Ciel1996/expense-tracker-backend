pub mod pot_api {
    use std::sync::Arc;
    use crate::api::{check_error, get_sub_claim, ApiResponse};
    use crate::currency_api::currency_api::CurrencyDTO;
    
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::http::request::Parts;
    use axum::Json;
    use expense_tracker_db::pots::pots::{NewPot, Pot, PotToUser};
    use expense_tracker_db::setup::DbPool;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use uuid::Uuid;
    use expense_tracker_db::users::users::User;
    use expense_tracker_services::currency_service::currency_service;
    use expense_tracker_services::currency_service::currency_service::CurrencyService;
    use expense_tracker_services::expense_service::expense_service;
    use expense_tracker_services::expense_service::expense_service::ExpenseService;
    use expense_tracker_services::pot_service::pot_service;
    use expense_tracker_services::pot_service::pot_service::PotService;
    use crate::expense_api::expense_api::{ExpenseDTO, NewExpenseDTO};
    use crate::user_api::user_api::UserDTO;

    /// Holds the App State for the PotAPI.
    pub struct PotApiState {
        pot_service: PotService,
        currency_service: CurrencyService,
        expense_service: ExpenseService
    }

    /// Registers all functions of the Pot API.
    pub fn register(pool: DbPool) -> OpenApiRouter {
        let shared_state = Arc::new(PotApiState {
            pot_service: pot_service::new_service(pool.clone()),
            currency_service: currency_service::new_service(pool.clone()),
            expense_service: expense_service::new_service(pool.clone())
        });

        OpenApiRouter::new()
            .routes(routes!(create_pot))
            .routes(routes!(get_pots))
            .routes(routes!(add_expense))
            .routes(routes!(get_pot_expenses))
            .routes(routes!(add_user_to_pot))
            .routes(routes!(remove_user_from_pot))
            .routes(routes!(delete_pot))
            .with_state(shared_state)
    }

    /// DTO used when working with existing Pots.
    #[derive(ToSchema, Serialize)]
    pub struct PotDTO {
        id: i32,
        owner_id: Uuid,
        name: String,
        default_currency: CurrencyDTO,
        users : Vec<UserDTO>
    }

    impl PotDTO {
        /// Creates a new PotDTO from a db Pot.
        pub fn from(pot: Pot, default_currency: CurrencyDTO, users: Vec<UserDTO>) -> Self {
            PotDTO {
                id: pot.id(),
                owner_id: pot.owner_id(),
                name: pot.name().to_string(),
                default_currency,
                users
            }
        }

        /// Create a vec<PotDTO> from a vec<Pot>.
        pub fn from_vec(pot_vec: Vec<(Pot, Vec<User>)>, currency_vec: Vec<CurrencyDTO>) -> Vec<Self> {
            let mut dtos: Vec<PotDTO> = vec![];

            for pot in pot_vec {
                let pot_currency = currency_vec
                    .iter()
                    .find(|c| c.id() == pot.0.default_currency_id());

                if let Some(pot_currency) = pot_currency {
                    dtos.push(
                        PotDTO::from(
                            pot.0,
                            (*pot_currency).clone(),
                            UserDTO::from_vec(pot.1)
                        ))
                }
            }

            dtos
        }
    }

    /// DTO used when creating a new Pot.
    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewPotDTO {
        name: String,
        default_currency_id: i32,
    }

    impl NewPotDTO {
        /// Converts the DTO to the db object.
        fn to_db(&self, owner_id: Uuid) -> NewPot {
            NewPot::new(owner_id, self.name.clone(), self.default_currency_id)
        }
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct AddUserToPotDTO {
        user_id: Uuid,
    }

    impl AddUserToPotDTO {
        pub fn user_id(&self) -> Uuid {
            self.user_id
        }
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct RemoveUserFromPotDTO {
        user_id: Uuid,
    }

    impl RemoveUserFromPotDTO {
        pub fn user_id(&self) -> Uuid {
            self.user_id
        }
    }

    /// Creates a pot from the given DTO for the bearer.
    #[utoipa::path(
        post,
        path = "/pots",
        tag = "Pots",
        responses(
            (status = 201, description = "The pot has been created", body = PotDTO),
        ),
        request_body = NewPotDTO,
        security(
            ("bearer" = [])
        )
    )]
    pub async fn create_pot(
        State(pot_api_state): State<Arc<PotApiState>>,
        parts : Parts,
        Json(new_pot): Json<NewPotDTO>
    ) -> Result<ApiResponse<PotDTO>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        let result = pot_api_state.pot_service
            .create_pot(new_pot.to_db(subject_id))
            .await
            .map_err(check_error)?;

        Ok((StatusCode::CREATED, Json(PotDTO::from(
            result.0,
            CurrencyDTO::from(result.1),
            UserDTO::from_vec(result.2)
        ))))
    }

    /// Gets the list of all pots the bearer can view.
    #[utoipa::path(
        get,
        path = "/pots",
        tag = "Pots",
        responses(
            (status = 200, description = "The list of known pots.", body = Vec<PotDTO>)
        ),
        security(
                ("bearer" = [])
        )
    )]
    pub async fn get_pots(
        State(pot_api_state): State<Arc<PotApiState>>,
        parts : Parts,
    ) -> Result<ApiResponse<Vec<PotDTO>>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        let loaded_pots = pot_api_state
            .pot_service
            .get_pots(subject_id)
            .await
            .map_err(check_error)?;

        let all_currencies = pot_api_state
            .currency_service
            .get_currencies()
            .await
            .map_err(check_error)?;

        let all_currencies = CurrencyDTO::from_vec(all_currencies);

        Ok((StatusCode::OK, Json(PotDTO::from_vec(loaded_pots, all_currencies))))
    }

    /// Adds the given user to the pot, if Bearer is the owner of that pot.
    #[utoipa::path(
            put,
            path = "/pots/{pot_id}",
            tag = "Pots",
            responses(
                (status = 200, description = "The user has successfully been added to the pot"),
                (status = 403, description = "The user could not be added due to the caller not being the owner of the given pot."),
                (status = 409, description = "The user was already added to the pot."),
                (status = 500, description = "An internal server error occurred")
            ),
            request_body = AddUserToPotDTO,
            params(
                ("pot_id" = i32, Path, description = "Pot database id for the pot.  ")
            ),
            security(
                    ("bearer" = [])
            )
        )]
    pub async fn add_user_to_pot(
        State(pot_api_state) : State<Arc<PotApiState>>,
        Path(pot_id) : Path<i32>,
        part : Parts,
        Json(add_user_to_pot_dto) : Json<AddUserToPotDTO>
    ) -> Result<ApiResponse<String>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&part)?;
        let new_user_id = add_user_to_pot_dto.user_id();

        let result = pot_api_state
            .pot_service
            .add_user_to_pot(PotToUser::new(pot_id, new_user_id), subject_id)
            .await
            .map_err(check_error)?;

        if !result {
            return Ok(
                (
                    StatusCode::FORBIDDEN,
                    Json(format!("User {} could not be added to pot {}", new_user_id, pot_id))
                ));
        }

        Ok((
            StatusCode::OK,
            Json(format!("User {} successfully added to pot {}", new_user_id, pot_id))
            ))
    }

    /// Removes the given user to the pot, if Bearer is the owner of that pot.
    #[utoipa::path(
            put,
            path = "/pots/{pot_id}/remove_user",
            tag = "Pots",
            responses(
                (status = 204, description = "The user has successfully been removed from the pot"),
                (status = 403, description = "The user could not be removed due to the caller not being the owner of the given pot."),
                (status = 404, description = "The user is not part of the pot."),
                (status = 500, description = "An internal server error occurred")
            ),
            request_body = RemoveUserFromPotDTO,
            params(
                ("pot_id" = i32, Path, description = "Pot database id for the pot.  ")
            ),
            security(
                    ("bearer" = [])
            )
    )]
    pub async fn remove_user_from_pot(
        State(pot_api_state) : State<Arc<PotApiState>>,
        Path(pot_id) : Path<i32>,
        part : Parts,
        Json(remove_user_from_pot_dto) : Json<RemoveUserFromPotDTO>
    ) -> Result<ApiResponse<String>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&part)?;
        let new_user_id = remove_user_from_pot_dto.user_id();

        let result = pot_api_state
            .pot_service
            .remove_user_from_pot(PotToUser::new(pot_id, new_user_id), subject_id)
            .await
            .map_err(check_error)?;

        if !result {
            return Ok(
                (
                    StatusCode::FORBIDDEN,
                    Json(format!("User {} could not be removed from pot {}", new_user_id, pot_id))
                ));
        }

        Ok((
            StatusCode::NO_CONTENT,
            Json(format!("User {} successfully removed from pot {}", new_user_id, pot_id))
        ))
    }

    /// Adds a new expense in the name of the user from the Bearer token to the pot with the given
    /// pot_id if it exists.
    #[utoipa::path(
        post,
        path = "/pots/{pot_id}",
        tag = "Pots",
        responses(
            (
                status = 201,
                description = "Indicates that the expense has been created for the given pot.",
                body = ExpenseDTO
            ),
            (
                status = 404,
                description = "Indicates that the pot for this expense does not exist."
            )
        ),
        request_body = NewExpenseDTO,
        params(
            ("pot_id" = i32, Path, description = "Pot database id for the pot.  ")
        ),
        security(
            ("bearer" = [])
        )
    )]
    pub async fn add_expense(
        State(pot_api_state): State<Arc<PotApiState>>,
        Path(pot_id): Path<i32>,
        part : Parts,
        Json(new_expense): Json<NewExpenseDTO>,
    ) -> Result<ApiResponse<ExpenseDTO>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&part)?;

        let loaded_pot = pot_api_state.
            pot_service
            .get_pot_by_id(pot_id, subject_id)
            .await
            .map_err(check_error)?;

        let expense_splits_result = pot_api_state
            .expense_service
            .create_expense(
                new_expense.to_db(loaded_pot.id(), subject_id),
                new_expense.splits_to_new_db()
            )
            .await
            .map_err(check_error)?;

        let expense = expense_splits_result.0;
        let splits = expense_splits_result.1;
        let currency = expense_splits_result.2;

        Ok((StatusCode::CREATED,Json(ExpenseDTO::from(expense, currency, splits, subject_id))))
    }

    /// Gets the sum of all expenses for the given user of the given pot.
    #[utoipa::path(
        get,
        path = "/pots/{pot_id}",
        tag = "Pots",
        responses(
            (
                status = 200,
                description = "The expenses for the pot with the given id",
                body = Vec<ExpenseDTO>
            ),
            (
                status = 404,
                description = "Indicates that the desired pot does not exists"
            )
        ),
        params(
            ("pot_id" = i32, Path, description = "Pot database id for the pot.  ")
        ),
        security(
            ("bearer" = [])
        )
    )]
    pub async fn get_pot_expenses(
        State(pot_api_service) : State<Arc<PotApiState>>,
        Path(pot_id): Path<i32>,
        parts : Parts
    ) -> Result<ApiResponse<Vec<ExpenseDTO>>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        let result = pot_api_service
            .expense_service
            .get_expenses_by_pot_id(pot_id, subject_id)
            .await
            .map_err(check_error)?;

        Ok((StatusCode::OK, Json(ExpenseDTO::from_vec(result, subject_id))))
    }

    /// Deletes the given pot if it does not contain any outstanding expenses and the caller
    /// is the pot's owner.
    #[utoipa::path(
        delete,
        path = "/pots/{pot_id}",
        tag = "Pots",
        responses(
            (
                status = 204,
                description = "The pot has been deleted."
            ),
            (
                status = 403,
                description = "Indicates that the user is not authorized to delete the given pot."
            ),
            (
                status = 404,
                description = "Indicates that the desired pot does not exists."
            ),
            (
                status = 409,
                description = "Indicates that the desired pot can't be deleted."
            )
        ),
        params(
            ("pot_id" = i32, Path, description = "Pot database id for the pot.  ")
        ),
        security(
            ("bearer" = [])
        )
    )]
    pub async fn delete_pot(
        State(pot_api_service) : State<Arc<PotApiState>>,
        Path(pot_id): Path<i32>,
        parts : Parts
    ) -> Result<ApiResponse<String>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        pot_api_service
            .pot_service
            .try_delete_pot(pot_id, subject_id)
            .await
            .map_err(check_error)?;

        Ok((StatusCode::NO_CONTENT, Json(format!("Deleted pot {}", pot_id))))
    }
}
