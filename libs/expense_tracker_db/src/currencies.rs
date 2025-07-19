pub mod currencies {
    use diesel::{Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};

    use crate::schema::currencies;

    /// Defines a currency with associated attributes.
    #[derive(Serialize, Selectable, Queryable, Clone)]
    #[diesel(table_name = currencies)]
    pub struct Currency {
        id: i32,
        name: String,
        symbol: String,
    }

    impl Currency {
        pub fn id(&self) -> i32 {
            self.id
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn symbol(&self) -> &str {
            &self.symbol
        }
    }

    /// This struct is used to define a new currency.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = currencies)]
    pub struct NewCurrency {
        name: String,
        symbol: String,
    }

    impl NewCurrency {
        /// Create a new NewCurrency struct instance.
        pub fn new(name: String, symbol: String) -> Self {
            NewCurrency { name, symbol }
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn symbol(&self) -> &str {
            &self.symbol
        }
    }
}
