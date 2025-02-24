use degen_sql::db::postgres::models::model::PostgresModelError;
use degen_sql::db::postgres::postgres_db::Database;
use chrono::{DateTime, Utc};
use tokio_postgres::Error as PostgresError;


#[derive(Clone)]
pub struct NetworkData {
    pub id: i32,  // Serial in PostgreSQL auto-increments, so it's generally represented as i32 in Rust
    pub chain_id: i64,
    pub latest_block_number: i64,
   // pub created_at: DateTime<Utc>,
}


impl NetworkData {
    pub async fn insert( 
        chain_id: i64,
        latest_block_number: i64,
          psql_db: &mut Database,
    ) -> Result<Self, PostgresModelError> {
        let statement =  "INSERT INTO network_data (chain_id, latest_block_number) VALUES ($1, $2) RETURNING id " ;
        let row = psql_db.query_one (&statement, &[&chain_id, &latest_block_number]).await?;

        Ok(Self {
            id: row.get(0),
            chain_id,
            latest_block_number,
            
        })
    }


       pub async fn insert_or_update(
        chain_id: i64,
        latest_block_number: i64,
        psql_db: &mut Database,
    ) -> Result<Self, PostgresModelError> {
        // SQL statement that handles both insert and update
        let statement = "INSERT INTO network_data (chain_id, latest_block_number) \
                         VALUES ($1, $2) \
                         ON CONFLICT (chain_id) \
                         DO UPDATE SET latest_block_number = EXCLUDED.latest_block_number \
                         RETURNING id";

        // Execute the query
        let row = psql_db.query_one (&statement, &[&chain_id, &latest_block_number]).await?;

        // Return the updated/inserted row
        Ok(Self {
            id: row.get(0),
            chain_id,
            latest_block_number,
        })
    }


    pub async fn find_one_by_chain_id(
        
        chain_id: i64,
          psql_db: &mut Database,
    ) -> Result<Self, PostgresModelError> {
        let statement =  "SELECT id, latest_block_number  FROM network_data WHERE chain_id = $1 ORDER BY created_at DESC LIMIT 1" ;
        let row = psql_db.query_one (&statement, &[&chain_id]).await?;

        Ok(Self {
            id: row.get(0),
            chain_id,
            latest_block_number: row.get(1),
            
        })
    }
}
