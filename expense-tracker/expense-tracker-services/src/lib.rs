use std::fmt::{Display, Formatter};

pub mod health_service;
pub mod user_service;
pub mod currency_service;
pub mod pot_service;
pub mod expense_service;

/// Internal error handling.
fn internal_error<E>(err: E) -> String
where
    E: std::error::Error,
{
    err.to_string()
}

/// Internal error handling if working with nested function calls.
fn internal_error_str(err: String) -> String
{
    err
}

#[derive(Debug)]
pub struct NotFoundError {
    message: String
}

impl Display for NotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for NotFoundError {}

fn not_found_error(message : String) -> NotFoundError {
    NotFoundError{
        message,
    }
}

#[derive(Debug)]
pub struct InternalError {
    message : String,
}

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for InternalError {}

// TODO: rename
fn internal_error_new<E>(err : E) -> InternalError {
    InternalError {
        message: err.to_string()
    }
}