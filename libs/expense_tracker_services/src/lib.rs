use std::fmt::{Display, Formatter};
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;
use diesel::result::Error;
use crate::cron_manager_service::cron_manager_service::CronManagerService;

pub mod currency_service;
pub mod expense_service;
pub mod health_service;
pub mod pot_service;
pub mod user_service;
pub mod template_service;
pub mod cron_manager_service;

/// We only need one instance of the cron manager service, so we use a static variable.
/// Making it pseudo singleton. Usinc Arc to ensure that only one instance exists at any time.
static CRON_MANAGER_SERVICE: LazyLock<Arc<Mutex<CronManagerService>>>
    = LazyLock::new(|| Arc::new(Mutex::new(CronManagerService::new())));

#[derive(Debug)]
/// An enumeration defining all errors of the application.
pub enum ExpenseError {
    /// Indicates that the requested resource could not be found.
    NotFound(String),
    /// Indicates that the requesting user does NOT have the authorization for the action.
    Forbidden(String),
    /// Indicates an unspecific error.
    Internal(String),
    /// Indicates a conflict, thus resulting in cancellation of the task.
    Conflict(String),
    /// Indicates that the resource is locked, most likely due to it being archived.
    Locked(String),
    /// Indicates that there was an error with the configuration of a cron job.
    CronConfigError(String),
}

/// Produces a `NotFound` from the given `err`.
fn not_found_error<E>(err: E) -> ExpenseError
where
    E: std::error::Error,
{
    ExpenseError::NotFound(err.to_string())
}

/// Produces a `Internal` from the given `err`.
fn internal_error<E>(err: E) -> ExpenseError
where
    E: std::error::Error,
{
    ExpenseError::Internal(err.to_string())
}

/// A helper used when unwrapping in case of an error.
fn check_error(err: ExpenseError) -> ExpenseError {
    err
}

impl From<ExpenseError> for Error {
    fn from(value: ExpenseError) -> Self {
        match value {
            ExpenseError::NotFound(_) => Error::NotFound,
            _ => panic!("Could not handle ExpenseError {:?}", value),
        }
    }
}

impl Display for ExpenseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
