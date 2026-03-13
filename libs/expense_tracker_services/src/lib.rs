use cron_tab::Cron;
use diesel::internal::derives::multiconnection::chrono::Local;
use diesel::result::Error;

pub mod currency_service;
pub mod expense_service;
pub mod health_service;
pub mod pot_service;
pub mod user_service;
pub mod template_service;
mod background_service;

// TODO: this works and prints out 'Hello, world!' 10 times!
// Now we need to setup a service that sets up cron jobs which then run at midnight on the CreateAt date for each PotTemplate!
// Additionally we need to make sure that the user can specify template parameters like {day}.{month}.{year} and that the cron job runs at the specified time
pub fn cron_test() {
    let mut test_cron = Cron::new(Local);

    test_cron.add_fn("* * * * * *", || {
        println!("Hello, world!");
    }).unwrap();

    test_cron.start();
    std::thread::sleep(std::time::Duration::from_secs(10));
    test_cron.stop();
}

#[derive(Debug)]
/// An enumeration defining all errors of the application.
pub enum ExpenseError {
    /// Indicates that the requested resource could not be found.
    NotFound(String),
    /// Indicates that the requesting user does NOT have the authorization for the action.
    Forbidden(String),
    /// Indicates an unspecific error.
    Internal(String),
    /// Indicates a conflict, thus resulting in cancelation of the task.
    Conflict(String),
    /// Indicates that the resource is locked, most likely due to it being archived.
    Locked(String),
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
