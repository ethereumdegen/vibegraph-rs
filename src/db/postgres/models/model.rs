

 

use tokio_postgres::Error as PostgresError;
use serde_json::Error as SerdeJsonError;

pub trait Model {} 



#[derive(Debug,thiserror::Error)]
pub enum PostgresModelError {
    #[error(transparent)]
    Postgres(#[from] PostgresError),

    #[error(transparent)]
    SerdeJson(#[from] SerdeJsonError),

    #[error("Error converting from hex")]
    AddressParseError 
}
 