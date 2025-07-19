/// Module containing health_service relevant code.
pub mod health_service {

    /// The PingHealthService
    #[derive(Clone)]
    pub struct PingHealthService;

    impl PingHealthService {
        /// Returns "Pong".
        pub fn ping(&self) -> String {
            "Pong".to_string()
        }
    }

    /// Returns a new struct which implements the HealthService trait.
    pub fn new_service() -> PingHealthService {
        PingHealthService {}
    }
}
