pub mod expense_api {
    use axum::body::Body;
    use axum::extract::{Path, State};
    use axum::http::Request;
    use axum::Json;
    use hyper::StatusCode;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use uuid::{uuid, Uuid};
    use expense_tracker_db::currencies::currencies::Currency;
    use expense_tracker_db::expenses::expenses::{Expense, NewExpense};
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::splits::splits::{NewExpenseSplit, Split};
    use expense_tracker_services::expense_service;
    use expense_tracker_services::expense_service::expense_service::{ExpenseService, JoinedExpense};
    use crate::api::{check_error, get_sub_claim, ApiResponse};
    use crate::currency_api::currency_api::CurrencyDTO;

    /// DTO used when working with existing Expenses.
    #[derive(ToSchema, Serialize)]
    pub struct ExpenseDTO {
        id: i32,
        pot_id: i32,
        owner_id: Uuid,
        description: String,
        currency: CurrencyDTO,
        splits: Vec<SplitDTO>,
        /// If negative: you have to pay `owner_id` this amount of money, otherwise
        /// you can expect others to pay you the given amount.
        sum: f64
    }

    impl ExpenseDTO {
        pub fn from(expense: Expense, currency: Currency, splits: Vec<Split>) -> Self {
            Self {
                id: expense.id(),
                description: expense.description().to_string(),
                pot_id: expense.pot_id(),
                currency: CurrencyDTO::from(currency),
                owner_id: expense.owner_id(),
                splits: SplitDTO::from_vec_split(splits.clone()),
                // TODO: viewer_id (1) must be gathered from oidc token!
                sum: get_sum(expense.owner_id(), uuid!("ddc96061-81da-489c-8c97-0a578079bd43"), splits)
            }
        }

        pub fn from_vec(expenses : Vec<JoinedExpense>) -> Vec<Self> {
            let mut dtos: Vec<ExpenseDTO> = vec!();

            for joined_expense in expenses {
                let expense = joined_expense.0;
                let splits = joined_expense.1;
                let currency = joined_expense.2;

                dtos.push(ExpenseDTO::from(expense, currency, splits))
            }

            dtos
        }
    }

    /// Gets the sum the `viewer_id`'s user owes the `expense_owner_id`'s user for the given
    /// `Vec<Split>`.
    pub fn get_sum(expense_owner_id: Uuid, viewer_id : Uuid, splits: Vec<Split>) -> f64 {
        let mut sum = 0.0;

        for split in splits {
            if !split.is_paid()
                && split.user_id().ne(&expense_owner_id) {

                // we only care about the money that is owed OR that WE owe the expense_owner!
                if expense_owner_id == viewer_id {
                    sum += split.amount();
                } else if split.user_id().eq(&viewer_id) {
                    sum -= split.amount();
                }
            }
        }

        sum
    }

    /// DTO used when working with splits.
    #[derive(Clone, ToSchema, Serialize, Deserialize)]
    pub struct SplitDTO {
        user_id: Uuid,
        amount: f64,
        /// Is `true` by default, if the Split's `user_id` == the expenses `owner_id`! Otherwise
        /// `false`. This is because the owner of an expense has already paid their part!
        is_paid: bool,
    }

    impl SplitDTO {
        /// Turns this SplitDTO into a db NewExpenseSplit.
        fn to_new_db(&self) -> NewExpenseSplit {
            NewExpenseSplit::new(self.user_id, self.amount, self.is_paid)
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
        owner_id: Uuid,
        description: String,
        currency_id: i32,
        splits: Vec<SplitDTO>,
    }

    impl NewExpenseDTO {
        /// Turns this NewExpenseDTO into a NewExpense.
        pub(crate) fn to_db(&self, owning_pot_id: i32) -> NewExpense {
            NewExpense::new(
                self.owner_id,
                owning_pot_id,
                self.description.clone(),
                self.currency_id,
            )
        }

        /// Turns the `Vec<SplitDTO>` into a `Vec<NewExpenseSplit>`.
        pub(crate) fn splits_to_new_db(&self) -> Vec<NewExpenseSplit> {
            let mut splits = vec!();

            for split in &self.splits {
                let db_split = split.to_new_db();
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

    pub fn register(pool: DbPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(get_expense_by_id))
            .with_state(expense_service::expense_service::new_service(pool))
    }

    /// Gets the expense with the given id. Returns 404 if no expense with the given id exists
    /// or the bearer has no access to it.
    #[utoipa::path(
        get,
        path = "/expenses/{expense_id}",
        tag = "Expenses",
        responses(
            (
                status = 200,
                description = "The Expense with the given id.",
                body = Vec<ExpenseDTO>
            ),
            (
                status = 404,
                description = "Indicates that the desired Expense does not exists"
            )
        ),
        params(
            ("expense_id" = i32, Path, description = "Expense database id for the Expense.  ")
        ),
        security(
            ("bearer" = [])
        )
    )]
    pub async fn get_expense_by_id(
        State(service) : State<ExpenseService>,
        Path(expense_id) : Path<i32>,
        request: Request<Body>
    ) -> Result<ApiResponse<ExpenseDTO>, ApiResponse<String>> {
        let (parts, _) = request.into_parts();
        let subject_id = get_sub_claim(&parts)?;

        let expense = service
            .get_expense_by_id(expense_id, subject_id)
            .await
            .map_err(check_error)?;

        Ok((StatusCode::OK, Json(ExpenseDTO::from(expense.0, expense.2, expense.1))))
    }
}

#[cfg(test)]
mod tests {
    use uuid::{uuid, Uuid};
    use expense_tracker_db::splits::splits::Split;
    use crate::expense_api::expense_api::get_sum;

    const USER_ONE : Uuid = uuid!("e6be621a-ec2d-48f3-8027-0d34cf5cbe40");
    const USER_TWO : Uuid = uuid!("729c270f-74d1-436f-aa46-4fe6a3dcb460");
    const USER_THREE : Uuid = uuid!("7ec9119b-10a2-48b8-be5f-d0f8ba9aad8d");
    const USER_FOUR : Uuid = uuid!("01913042-053a-4cb2-846d-4b58153185b8");
    const USER_FIVE : Uuid = uuid!("60755204-45d7-4f0d-96e6-bf61e6f3feda");
    const OTHER_USER : Uuid = uuid!("95f5222d-1b50-407e-b13e-8213d39764cd");

    /// First case: Expense_Owner is the viewer, so sum should be 0. Only one split
    #[test]
    fn get_sum_test_expense_owner_is_viewer_one_split() {
        let splits = vec![
            Split::new(1, USER_ONE, 42.0, true),
        ];

        let sum = get_sum(USER_ONE, USER_ONE, splits);

        assert_eq!(sum, 0.0);
    }

    /// Second case: Expense_Owner is the viewer, so sum should be positive sum of
    /// all splits (except own). Multiple splits, different debtors. Nothing paid.
    #[test]
    fn get_sum_test_expense_owner_is_viewer_multiple_splits_nothing_paid() {
        let splits = vec![
            Split::new(1, USER_ONE, 42.0, true),
            Split::new(1, USER_TWO, 42.0, false),
            Split::new(1, USER_THREE, 42.0, false),
        ];

        let sum = get_sum(USER_ONE, USER_ONE, splits);

        assert_eq!(sum, 84.0);
    }

    /// Third case: Expense_Owner is the viewer, so sum should be positive sum of
    /// all splits (except own). Multiple splits, different debtors. Some paid.
    #[test]
    fn get_sum_test_expense_owner_is_viewer_multiple_splits_some_paid() {
        let splits = vec![
            Split::new(1, USER_ONE, 42.0, true),
            Split::new(1, USER_TWO, 42.0, false),
            Split::new(1, USER_THREE, 42.0, false),
            Split::new(1, USER_FOUR, 42.0, true),
            Split::new(1, USER_FIVE, 42.0, true),
        ];

        let sum = get_sum(USER_ONE, USER_ONE, splits);

        assert_eq!(sum, 84.0);
    }

    /// Fourth case: Expense_Owner is not the viewer, so sum should be negative sum of
    /// owned splits (except own). Multiple splits, different debtors. Nothing paid.
    #[test]
    fn get_sum_test_expense_owner_is_not_viewer_multiple_splits_nothing_paid() {
        let splits = vec![
            Split::new(1, USER_ONE, 42.0, true),
            Split::new(1, USER_TWO, 42.0, false),
            Split::new(1, USER_THREE, 42.0, false),
        ];

        let sum = get_sum(USER_ONE, USER_TWO, splits);

        assert_eq!(sum, -42.0);
    }

    /// Fifth case: Expense_Owner is not the viewer, so sum should be negative sum of
    /// owned splits (except own). Multiple splits, different debtors. Viewer paid.
    #[test]
    fn get_sum_test_expense_owner_is_not_viewer_multiple_splits_viewer_paid() {
        let splits = vec![
            Split::new(1, USER_ONE, 42.0, true),
            Split::new(1, USER_TWO, 42.0, true),
            Split::new(1, USER_THREE, 42.0, false),
        ];

        let sum = get_sum(USER_ONE, USER_TWO, splits);

        assert_eq!(sum, 0.0);
    }

    /// Sixth case: Expense_Owner is not the viewer, so sum should be negative sum of
    /// owned splits (except own). Multiple splits, different debtors. Viewer paid some.
    #[test]
    fn get_sum_test_expense_owner_is_not_viewer_multiple_splits_viewer_some_paid() {
        let splits = vec![
            Split::new(1, USER_ONE, 42.0, true),
            Split::new(1, USER_TWO, 42.0, true),
            Split::new(1, USER_TWO, 42.0, false),
            Split::new(1, USER_TWO, 42.0, false),
            Split::new(1, USER_THREE, 42.0, false),
        ];

        let sum = get_sum(USER_ONE, USER_TWO, splits);

        assert_eq!(sum, -84.0);
    }

    /// Eighth case: Expense_Owner is not the viewer, viewer does not owe any money.
    /// Multiple splits, different debtors. Viewer paid some.
    #[test]
    fn get_sum_test_expense_owner_is_not_viewer_multiple_splits_viewer_some_paid_view_is_not_debtor()
    {
        let splits = vec![
            Split::new(1, USER_ONE, 42.0, true),
            Split::new(1, USER_TWO, 42.0, true),
            Split::new(1, USER_TWO, 42.0, false),
            Split::new(1, USER_TWO, 42.0, false),
            Split::new(1, USER_THREE, 42.0, false),
        ];

        let sum = get_sum(USER_ONE, OTHER_USER, splits);

        assert_eq!(sum, 0.0);
    }
}