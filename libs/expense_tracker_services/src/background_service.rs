pub mod background_service {
    use cron_tab::AsyncCron;
    use diesel::internal::derives::multiconnection::chrono::{DateTime, Datelike, Local, Timelike, Utc};
    use uuid::Uuid;

    struct CronJobWrapper {
        async_cron: AsyncCron<Utc>,
        id : Uuid
    }

    pub struct BackgroundService {
        cron_jobs : Vec<CronJobWrapper>
    }

    impl BackgroundService {
        pub fn new() -> Self {
            BackgroundService {
                cron_jobs : Vec::new()
            }
        }

        /// Adds a cron job to the list of cron jobs and starts it.
        pub fn add_cron_job_at(
            &mut self,
            utc_date_time : DateTime<Utc>,
            function : Box<dyn Fn() + Send + Sync>) -> Uuid {
            let id = Uuid::new_v4();
            let mut cron_job = AsyncCron::new(Utc);

            // cron_job.add_fn(self.to_cron(utc_date_time), function);

            let cron_job_wrapper = CronJobWrapper {
                async_cron: cron_job,
                id
            };

            self.cron_jobs.push(cron_job_wrapper);

            id
        }

        /// Converts a DateTime<Utc> to a cron string.
        pub(crate) fn to_cron(&self, utc_date_time: DateTime<Utc>) -> String {
            utc_date_time.format(&format!("{} {} {} {} {} * {}",
                                          utc_date_time.second(),
                                          utc_date_time.minute(),
                                          utc_date_time.hour(),
                                          utc_date_time.day(),
                                          utc_date_time.month(),
                                          utc_date_time.year()))
                .to_string()
        }
    }
}

#[cfg(test)]
mod test {
    use diesel::internal::derives::multiconnection::chrono;
    use diesel::internal::derives::multiconnection::chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
    use crate::background_service::background_service::BackgroundService;

    #[test]
    fn test_to_cron_only_ymd() {
        // Arrange
        let utc_date_time = Utc.with_ymd_and_hms(2022, 1, 1, 0, 0, 0);
        let expected_cron = "0 0 0 1 1 * 2022".to_string();
        let service = BackgroundService::new();

        // Act & Assert
        let actual_cron = match utc_date_time {
            chrono::LocalResult::Single(datetime) => service.to_cron(datetime),
            chrono::LocalResult::None => panic!("Invalid UTC date-time provided"),
            chrono::LocalResult::Ambiguous(_, _) => panic!("Ambiguous UTC date-time provided"),
        };
        assert_eq!(expected_cron, actual_cron);
    }

    #[test]
    fn test_to_cron_ymd_and_hms() {
        // Arrange
        let utc_date_time = Utc.with_ymd_and_hms(2022, 1, 1, 5, 2, 4);
        let expected_cron = "4 2 5 1 1 * 2022".to_string();
        let service = BackgroundService::new();

        // Act & Assert
        let actual_cron = match utc_date_time {
            chrono::LocalResult::Single(datetime) => service.to_cron(datetime),
            chrono::LocalResult::None => panic!("Invalid UTC date-time provided"),
            chrono::LocalResult::Ambiguous(_, _) => panic!("Ambiguous UTC date-time provided"),
        };
        assert_eq!(expected_cron, actual_cron);
    }
}