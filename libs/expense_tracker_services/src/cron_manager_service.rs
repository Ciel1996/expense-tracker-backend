pub mod cron_manager_service {
    use std::sync::Arc;
    use cron_tab::{AsyncCron, CronError};
    use diesel::internal::derives::multiconnection::chrono::Local;
    use log::error;

    pub(crate) struct CronJobWrapper {
        pub async_cron: AsyncCron<Local>,
        pub id : u32
    }

    pub(crate) struct CronManagerService {
        pub cron_jobs : Vec<CronJobWrapper>,
        next_id : u32
    }

    impl CronManagerService {
        pub fn new() -> Self {
            CronManagerService {
                cron_jobs : Vec::new(),
                next_id: 0
            }
        }

        /// Adds a cron job to the list of cron jobs and starts it.
        pub async fn add_cron_job(
            &mut self,
            cron_expression : &str,
            function : Box<dyn Fn() + Send + Sync>) -> Result<u32, CronError> {

            // set id to next_id and increment next_id, u32 should suffice,
            // as we are not expecting that many cron jobs to be scheduled at the same time
            let id = self.next_id;
            self.next_id += 1;

            // use local timezone so that the user is saved some headache
            let mut cron_job = AsyncCron::new(Local);

            let function = Arc::new(function);
            let job = cron_job.add_fn(cron_expression, {
                let function = Arc::clone(&function);
                move || {
                    let function = function.clone();
                    async move { (function)() }
                }
            }).await;

            if let Err(e) = job {
                error!("add_cron_job_at failed with parameters {} and error {}", cron_expression, e);
                return Err(e);
            }

            let cron_job_wrapper = CronJobWrapper {
                async_cron: cron_job,
                id
            };

            self.cron_jobs.push(cron_job_wrapper);

            Ok(id)
        }

        /// Removes a cron job from the list of cron jobs.
        pub async fn remove_cron_job(&mut self, id : u32) {
            for cron_job in self.cron_jobs.iter() {
                if cron_job.id == id {
                    // stop the cron job to avoid having a running cron job that is neither scheduled
                    // to run nor tracked by the CronManagerService
                    cron_job.async_cron.stop().await;
                }
            }

            // remove the cron job from the list of cron jobs
            self.cron_jobs.retain(|cron_job| cron_job.id != id);
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
    use crate::cron_manager_service::cron_manager_service::CronManagerService;

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
        assert_eq!(job_id, 0);

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

        assert_eq!(job_id_one.unwrap(), 0);

        let job_id_two = background_service
            .add_cron_job(every_first_of_month, function.clone())
            .await;

        assert_eq!(job_id_two.unwrap(), 1);

        let job_id_three = background_service
            .add_cron_job(every_first_of_month, function.clone())
            .await;

        assert_eq!(job_id_three.unwrap(), 2);

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

        background_service.remove_cron_job(job_id).await;

        assert_eq!(background_service.cron_jobs.len(), 1);

        background_service.remove_cron_job(job_id_two).await;

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

        background_service.remove_cron_job(job_id).await;

        assert_eq!(background_service.cron_jobs.len(), 1);

        background_service.remove_cron_job(job_id_two).await;

        assert_eq!(background_service.cron_jobs.len(), 0);
    }

}