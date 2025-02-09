pub mod pot_api {
    use crate::api::{internal_error, internal_error_new};
    use crate::currency_api::currency_api::CurrencyDTO;
    
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::Json;
    use diesel::{
       RunQueryDsl, SelectableHelper,
    };
    use expense_tracker_db::expenses::expenses::{Expense, NewExpense};
    use expense_tracker_db::pots::pots::{NewPot, Pot};
    use expense_tracker_db::schema as expense_tracker_db_schema;
    use expense_tracker_db::setup::DbConnectionPool;
    use expense_tracker_db::splits::splits::{NewSplit, Split};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use expense_tracker_services::currency_service::currency_service::CurrencyService;
    use expense_tracker_services::expense_service::expense_service::ExpenseService;
    use expense_tracker_services::pot_service::pot_service;
    use expense_tracker_services::pot_service::pot_service::PotService;

    // TODO: don't use a struct looking like this, try to get a "real" DI working.
    pub struct IDontLikeThis {
        pot_service: PotService,
        currency_service: CurrencyService,
        expense_service: ExpenseService
    }

    /// Registers all functions of the Pot API.
    pub fn register(pool: DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(create_pot))
            .routes(routes!(get_pots))
            .routes(routes!(add_expense))
            .with_state(pot_service::new_service(pool))
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
        pub fn from(pot: Pot, default_currency: CurrencyDTO) -> Self {
            PotDTO {
                id: pot.id(),
                owner_id: pot.owner_id(),
                name: pot.name().to_string(),
                default_currency,
            }
        }

        /// Create a vec<PotDTO> from a vec<Pot>.
        pub fn from_vec(pot_vec: Vec<Pot>, currency_vec: Vec<CurrencyDTO>) -> Vec<Self> {
            let mut dtos: Vec<PotDTO> = vec![];

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
            NewPot::new(self.owner_id, self.name.clone(), self.default_currency_id)
        }
    }

    /// DTO used when working with existing Expenses.
    #[derive(ToSchema, Serialize)]
    pub struct ExpenseDTO {
        id: i32,
        pot_id: i32,
        owner_id: i32,
        description: String,
        currency: CurrencyDTO,
        splits: Vec<SplitDTO>,
    }

    impl ExpenseDTO {
        fn from(expense: Expense, currency: CurrencyDTO, splits: Vec<SplitDTO>) -> Self {
            Self {
                id: expense.id(),
                description: expense.description().to_string(),
                pot_id: expense.pot_id(),
                currency,
                owner_id: expense.owner_id(),
                splits,
            }
        }
    }

    /// DTO used when working with splits.
    #[derive(Clone, ToSchema, Serialize, Deserialize)]
    pub struct SplitDTO {
        user_id: i32,
        amount: f64,
        is_paid: bool,
    }

    impl SplitDTO {
        /// Turns this SplitDTO into a db Split.
        fn to_new_db(&self, expense_id: i32) -> NewSplit {
            NewSplit::new(expense_id, self.user_id, self.amount, self.is_paid)
        }

        fn from(split : Split) -> Self {
            Self {
                user_id: split.user_id(),
                is_paid: split.is_paid(),
                amount: split.amount(),
            }
        }

        fn from_vec_split(splits : Vec<Split>) -> Vec<SplitDTO> {
            let mut dtos: Vec<SplitDTO> = vec![];

            for split in splits {
                dtos.push(SplitDTO::from(split))
            }

            dtos
        }
    }

    /// DTO used when creating a new expense for the given pot.
    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewExpenseDTO {
        owner_id: i32,
        description: String,
        currency_id: i32,
        splits: Vec<SplitDTO>,
    }

    impl NewExpenseDTO {
        /// Turns this NewExpenseDTO into a NewExpense.
        fn to_db(&self, owning_pot_id: i32) -> NewExpense {
            NewExpense::new(
                self.owner_id,
                owning_pot_id,
                self.description.clone(),
                self.currency_id,
            )
        }

        /// Turns the Vec<SplitDTO> into a Vec<NewSplit>.
        fn splits_to_new_db(&self, expense_id: i32) -> Vec<NewSplit> {
            let mut splits: Vec<NewSplit> = vec![];

            for split in &self.splits {
                let db_split = split.to_new_db(expense_id);
                splits.push(db_split);
            }

            splits
        }
    }

    impl Clone for NewExpenseDTO {
        fn clone(&self) -> Self {
            Self {
                owner_id: self.owner_id,
                description: self.description.clone(),
                currency_id: self.currency_id,
                splits: self.splits.clone()
            }
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
        State(service): State<PotService>,
        Json(new_pot): Json<NewPotDTO>,
    ) -> Result<Json<PotDTO>, (StatusCode, String)> {
        let result = service
            .create_pot(new_pot.to_db())
            .await
            .map_err(internal_error_new)?;

        Ok(Json(PotDTO::from(result.0, CurrencyDTO::from(result.1))))
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
        State(service): State<PotService>,
    ) -> Result<Json<Vec<PotDTO>>, (StatusCode, String)> {
        let loaded_pots = service.get_pots()
            .await
            .map_err(internal_error_new)?;

        let all_currencies = service
            .currency_service()
            .get_currencies()
            .await
            .map_err(internal_error_new)?;

        let all_currencies = CurrencyDTO::from_vec(all_currencies);

        Ok(Json(PotDTO::from_vec(loaded_pots, all_currencies)))
    }

    #[utoipa::path(
        post,
        path = "/pots/{pot_id}",
        tag = "Pots",
        responses(
            (status = 201, description = "Indicates that the expense has been created for the given pot.", body = ExpenseDTO),
            (status = 404, description = "Indicates that the pot for this expense does not exist.")
        ),
        request_body = NewExpenseDTO,
        params(
            ("pot_id" = i32, Path, description = "Pot database id for the pot.")
        )
    )]
    pub async fn add_expense(
        State(service): State<PotService>,
        State(expense_service) : State<ExpenseService>,
        Path(pot_id): Path<i32>,
        Json(new_expense): Json<NewExpenseDTO>,
    ) -> Result<Json<ExpenseDTO>, (StatusCode, String)> {
        // TODO: make sure that a 404 is returned when no pot with given id exists
        let loaded_pot = service
            .get_pot_by_id(pot_id)
            .await
            .map_err(internal_error_new)?;

        let new_expense_clone = new_expense.clone();
        let cloned_request_id = pot_id.clone();

        let res = conn
            .interact(move |conn| {
                diesel::insert_into(expense_tracker_db_schema::expenses::table)
                    .values(new_expense_clone.to_db(cloned_request_id))
                    .returning(Expense::as_returning())
                    .get_result::<Expense>(conn)
            })
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        let expense_id = res.id();
        let new_expense_clone = new_expense.clone();

        let splits = conn.interact(move |conn| {
            diesel::insert_into(expense_tracker_db_schema::expense_splits::table)
                .values(&new_expense_clone.splits_to_new_db(expense_id))
                .returning(Split::as_returning())
                .get_results::<Split>(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

        let currency = new_expense.currency_id;

        // TODO: get rid of Option
        let currency = service
            .currency_service()
            .get_currency_by_id(currency)
            .await
            .map_err(internal_error)?;

        let splits = SplitDTO::from_vec_split(splits);

        Ok(Json(ExpenseDTO::from(res, CurrencyDTO::from(currency), splits)))
    }
}
