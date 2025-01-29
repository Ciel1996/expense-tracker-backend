pub mod health_service;
pub mod user_service;


/// Internal error handling.
fn internal_error<E>(err: E) -> String
where
    E: std::error::Error,
{
    err.to_string()
}