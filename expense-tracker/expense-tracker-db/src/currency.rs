pub mod currencies {
    use diesel::{Insertable, Queryable, Selectable};
    use serde::{Deserialize, Serialize};
    use crate::schema::currencies;

    /// Defines a currency with associated attributes.
    #[derive(Serialize, Selectable, Queryable)]
    pub struct Currency {
        id: i32,
        name: String,
        symbol: String,
    }

    /// This struct is used to define a new currency.
    #[derive(Deserialize, Insertable)]
    #[diesel(table_name = currencies)]
    pub struct NewCurrency {
        name: String,
        symbol: String,
    }
}