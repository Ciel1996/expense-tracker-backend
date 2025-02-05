pub mod health_service;
pub mod user_service;
pub mod currency_service;

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