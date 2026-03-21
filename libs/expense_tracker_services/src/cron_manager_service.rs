pub mod cron_manager_service {
    use std::sync::Arc;
    use cron_tab::AsyncCron;
    use diesel::internal::derives::multiconnection::chrono::Local;
    use log::error;
    use uuid::Uuid;
    use crate::ExpenseError;

    /// Represents the UUID: 23bb4a46-89a1-4521-a77f-9b946dd87e66
    pub(crate) const ID_NAMESPACE: Uuid = Uuid::from_bytes([0x23,0xbb,0x4a,0x46,0x89,0xa1,0x45,0x21,0xa7,0x7f,0x9b,0x94,0x6d,0xd8,0x7e,0x66]);
    /// Represents the UUID: 58f7d6e0-fb17-45ba-9ba2-041052fbaeeb
    pub(crate) const CUSTOM_ID_NAMESPACE: Uuid = Uuid::from_bytes([0x58,0xf7,0xd6,0xe0,0xfb,0x17,0x45,0xba,0x9b,0xa2,0x04,0x10,0x52,0xfb,0xae,0xeb]);

    pub(crate) struct CronJobWrapper {
        pub async_cron: AsyncCron<Local>,
        pub id : Uuid
    }

    pub struct CronManagerService {
        pub(crate) cron_jobs : Vec<CronJobWrapper>
    }

    impl CronManagerService {

        pub fn new() -> Self {
            CronManagerService {
                cron_jobs : Vec::new()
            }
        }

        /// Adds a cron job to the list of cron jobs and starts it.
        /// This differs from add_cron_job in that it allows the user to specify the id of the cron job.
        pub async fn add_cron_job_with_id(
            &mut self,
            cron_expression : &str,
            function : Box<dyn Fn() + Send + Sync>,
            id : i32) -> Result<Uuid, ExpenseError> {
            let calculated_custom_id = Uuid::new_v5(&CUSTOM_ID_NAMESPACE, &id.to_be_bytes());

            // check if the id is already taken
            if self.cron_jobs.iter().any(|cron_job| cron_job.id == calculated_custom_id) {
                return Err(ExpenseError::CronConfigError(format!("ID {} is already taken", id)));
            }

            // Call the internal function to add the cron job
            // using the calculated custom id
            self.add_cron_job_internal(calculated_custom_id, cron_expression, function).await
        }

        async fn add_cron_job_internal(
            &mut self,
            id : Uuid,
            cron_expression : &str,
            function : Box<dyn Fn() + Send + Sync>) -> Result<Uuid, ExpenseError> {
            // use local timezone so that the user is saved some headache
            let mut cron_job = AsyncCron::new(Local);

            let function = Arc::new(function);
            let job = cron_job.add_fn(cron_expression, {
                let function = Arc::clone(&function);
                move || {
                    let function = function.clone();
                    async move { function() }
                }
            }).await;

            if let Err(e) = job {
                error!("add_cron_job failed with cron_expression {} and error {}", cron_expression, e);
                return Err(ExpenseError::CronConfigError(
                    format!(
                        "add_cron_job failed with cron_expression {} and error {}",
                        cron_expression,
                        e
                    )));
            }

            let cron_job_wrapper = CronJobWrapper {
                async_cron: cron_job,
                id
            };

            self.cron_jobs.push(cron_job_wrapper);

            Ok(id)
        }

        /// Adds a cron job to the list of cron jobs and starts it.
        pub async fn add_cron_job(
            &mut self,
            cron_expression : &str,
            function : Box<dyn Fn() + Send + Sync>) -> Result<Uuid, ExpenseError> {
            let id = Uuid::new_v5(&ID_NAMESPACE, Uuid::new_v4().as_bytes());
            self.add_cron_job_internal(id, cron_expression, function).await
        }

        /// Removes a cron job from the list of cron jobs.
        pub async fn remove_cron_job(&mut self, id : &Uuid) {
            for cron_job in self.cron_jobs.iter() {
                if cron_job.id.eq(id) {
                    // stop the cron job to avoid having a running cron job that is neither scheduled
                    // to run nor tracked by the CronManagerService
                    cron_job.async_cron.stop().await;
                }
            }

            // remove the cron job from the list of cron jobs
            self.cron_jobs.retain(|cron_job| cron_job.id.ne(id));
        }

        /// Runs all cron jobs that are scheduled to run.
        pub async fn run_cron_jobs(&mut self) {
            for cron_job in self.cron_jobs.iter_mut() {
                cron_job.async_cron.start().await;
            }
        }

        /// Stops all cron jobs that are scheduled to run.
        pub async fn stop_cron_jobs(&mut self) {
            for cron_job in self.cron_jobs.iter_mut() {
                cron_job.async_cron.stop().await;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use uuid::{Uuid, Version};
    use crate::cron_manager_service::cron_manager_service::{CronManagerService, CUSTOM_ID_NAMESPACE};

    #[tokio::test]
    async fn test_add_cron_job() {
        let mut background_service = CronManagerService::new();
        let every_first_of_month = "0 0 0 1 * *";
        let function = Box::new(|| { });

        let job_id = background_service
            .add_cron_job(every_first_of_month, function)
            .await;

        assert!(job_id.is_ok());

        let job_id = job_id.unwrap();
        // ensure that the id is a valid uuid, created from custom data and the namespace
        assert_eq!(job_id.get_version().unwrap(), Version::Sha1);

        let job_wrappers = background_service.cron_jobs;
        assert_eq!(job_wrappers.len(), 1);
        assert_eq!(job_wrappers[0].id, job_id);
    }

    #[tokio::test]
    async fn test_add_cron_job_multiple_times() {
        let mut background_service = CronManagerService::new();
        let every_first_of_month = "0 0 0 1 * *";
        let function = Box::new(|| { });

        let job_id_one = background_service
            .add_cron_job(every_first_of_month, function.clone())
            .await;

        let job_one_uuid = job_id_one.as_ref().unwrap();
        assert_eq!(job_one_uuid.get_version().unwrap(), Version::Sha1);

        let job_id_two = background_service
            .add_cron_job(every_first_of_month, function.clone())
            .await;

        let job_two_uuid = job_id_two.as_ref().unwrap();
        assert_eq!(job_two_uuid.get_version().unwrap(), Version::Sha1);
        assert_ne!(job_one_uuid, job_two_uuid);

        let job_id_three = background_service
            .add_cron_job(every_first_of_month, function.clone())
            .await;

        let job_three_uuid = job_id_three.as_ref().unwrap();
        assert_eq!(job_three_uuid.get_version().unwrap(), Version::Sha1);
        assert_ne!(job_one_uuid, job_three_uuid);
        assert_ne!(job_two_uuid, job_three_uuid);
    }

    #[tokio::test]
    async fn test_add_cron_job_with_id_multiple_times() {
        let mut background_service = CronManagerService::new();
        let every_first_of_month = "0 0 0 1 * *";
        let function = Box::new(|| { });

        let id_one = 42;
        let job_id_one = background_service
            .add_cron_job_with_id(every_first_of_month, function.clone(), id_one)
            .await;

        let job_one_uuid = job_id_one.as_ref().unwrap();
        let job_one_expected = Uuid::new_v5(&CUSTOM_ID_NAMESPACE, &id_one.to_be_bytes());
        assert_eq!(job_one_uuid.get_version().unwrap(), Version::Sha1);
        assert_eq!(job_one_uuid, &job_one_expected);

        let id_two = 99;
        let job_id_two = background_service
            .add_cron_job_with_id(every_first_of_month, function.clone(), id_two)
            .await;

        let job_two_uuid = job_id_two.as_ref().unwrap();
        let job_two_expected = Uuid::new_v5(&CUSTOM_ID_NAMESPACE, &id_two.to_be_bytes());
        assert_eq!(job_two_uuid.get_version().unwrap(), Version::Sha1);
        assert_eq!(job_two_uuid, &job_two_expected);

        let id_three = 2026;
        let job_id_three = background_service
            .add_cron_job_with_id(every_first_of_month, function.clone(), id_three)
            .await;

        let job_three_uuid = job_id_three.as_ref().unwrap();
        let job_three_expected = Uuid::new_v5(&CUSTOM_ID_NAMESPACE, &id_three.to_be_bytes());
        assert_eq!(job_three_uuid.get_version().unwrap(), Version::Sha1);
        assert_eq!(job_three_uuid, &job_three_expected);
    }

    #[tokio::test]
    async fn test_add_cron_job_with_same_id_multiple_times() {
        let mut background_service = CronManagerService::new();
        let every_first_of_month = "0 0 0 1 * *";
        let function = Box::new(|| { });

        let id_one = 42;
        let job_id_one = background_service
            .add_cron_job_with_id(every_first_of_month, function.clone(), id_one)
            .await;

        let job_one_uuid = job_id_one.as_ref().unwrap();
        let job_one_expected = Uuid::new_v5(&CUSTOM_ID_NAMESPACE, &id_one.to_be_bytes());

        assert_eq!(job_one_uuid, &job_one_expected);

        let id_two = 42;
        let job_id_two = background_service
            .add_cron_job_with_id(every_first_of_month, function.clone(), id_two)
            .await;

        assert!(job_id_two.is_err());
    }

    /// Note that this test has a high potential of being flaky.
    #[tokio::test]
    async fn test_run_cron_jobs_can_run() {
        let mut background_service = CronManagerService::new();
        let every_two_seconds = "*/2 * * * * *";

        let output_vec = Arc::new(Mutex::new(Vec::new()));
        let output_vec_clone = Arc::clone(&output_vec);
        let function = Box::new(move || {
            if let Ok(mut vec) = output_vec_clone.lock() {
                vec.push("Hello World!".to_string());
            }
        });

        // don't care about the result, just make sure that the function is called
        let _ = background_service
            .add_cron_job(every_two_seconds, function)
            .await;

        // run the cron jobs
        background_service.run_cron_jobs().await;

        tokio::time::sleep(std::time::Duration::from_secs(6)).await;

        // stop the cron jobs
        background_service.stop_cron_jobs().await;

        assert_eq!(output_vec.lock().unwrap().len(), 3);
        assert_eq!(output_vec.lock().unwrap()[0], "Hello World!");
        assert_eq!(output_vec.lock().unwrap()[1], "Hello World!");
        assert_eq!(output_vec.lock().unwrap()[2], "Hello World!");
    }

    #[tokio::test]
    async fn test_remove_cron_job() {
        let mut background_service = CronManagerService::new();
        let every_two_seconds = "*/2 * * * * *";
        let function = Box::new(|| {});
        let job_id = background_service
            .add_cron_job(every_two_seconds, function)
            .await.unwrap();

        let function = Box::new(|| {});
        let job_id_two = background_service
            .add_cron_job(every_two_seconds, function)
            .await.unwrap();

        assert_eq!(background_service.cron_jobs.len(), 2);

        background_service.remove_cron_job(&job_id).await;

        assert_eq!(background_service.cron_jobs.len(), 1);

        background_service.remove_cron_job(&job_id_two).await;

        assert_eq!(background_service.cron_jobs.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_cron_running_jobs() {
        let mut background_service = CronManagerService::new();
        let every_two_seconds = "*/2 * * * * *";
        let function = Box::new(|| {});
        let job_id = background_service
            .add_cron_job(every_two_seconds, function)
            .await.unwrap();

        let function = Box::new(|| {});
        let job_id_two = background_service
            .add_cron_job(every_two_seconds, function)
            .await.unwrap();

        background_service.run_cron_jobs().await;

        assert_eq!(background_service.cron_jobs.len(), 2);

        background_service.remove_cron_job(&job_id).await;

        assert_eq!(background_service.cron_jobs.len(), 1);

        background_service.remove_cron_job(&job_id_two).await;

        assert_eq!(background_service.cron_jobs.len(), 0);
    }

}