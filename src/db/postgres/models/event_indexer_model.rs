use tokio_postgres::Row;
use ethers::types::H256;
use ethers::types::{Address, U256, U64};

 

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use log::info;
use serde_json;
use std::str::FromStr;
use tokio::time::timeout;
use std::time::Duration;

use crate::event::ContractEvent;
use degen_sql::db::postgres::models::model::PostgresModelError;
use degen_sql::db::postgres::postgres_db::Database;




pub struct EventIndexer {
    pub id: u64,
    pub contract_name: String,
    pub contract_address: Address,
    pub chain_id: u64,
    pub start_block: u64,
    pub current_indexing_block: Option<u64>,
    pub synced: bool,
  //  pub created_at: DateTime<Utc>,
}

impl EventIndexer {


    pub fn from_row(row: &Row) -> Result<Self, PostgresModelError>{

          let contract_address = Address::from_str(&row.get::<_, String>("contract_address"))
                .map_err(|e| PostgresModelError::RowParseError(format!("Invalid contract address: {:?}", e).into()))?;



        Ok( Self{ 
        	     id: (row.get::<_, i64>("id")) as u64 , 
        	     contract_name: row.get("contract_name"),
                 contract_address  ,

                 chain_id: (row.get::<_, i64>("chain_id")) as u64 , 
                  start_block: (row.get::<_, i64>("start_block")) as u64 , 


                   current_indexing_block: (row.try_get::<_, i64>("current_indexing_block"))  .ok().map(|i| i as u64) , 
                    synced: (row.get::<_, bool>("synced"))   , 



 
            })


    }
    


}

pub struct EventIndexerModel {}

impl EventIndexerModel {


	pub async fn find_next_event_indexer(
	    offset_indexer_id:i64,
	    psql_db: &mut Database,
	) -> Result< EventIndexer, PostgresModelError> {
	    
	      let query = "
            SELECT id, contract_name, contract_address, chain_id, start_block, current_indexing_block, synced, created_at
            FROM event_indexers
            WHERE id > $1
            ORDER BY id ASC
            LIMIT 1;
        ";

        let row = psql_db.query_one_with_reconnect(query, &[&offset_indexer_id]).await?;
 	


 		EventIndexer::from_row(&row)
	    
	}
	     
	     


	    pub async fn insert_event_indexer(
        event: &ContractEvent,
        psql_db: &mut Database,
    ) -> Result<i32, PostgresModelError> {
        let contract_address = event.address.to_string();
        let contract_name = &event.name;

        let chain_id = event.chain_id as  i64 ;
        let start_block = event.block_number.map(|num| num.as_u64() as i64);

        let result = psql_db.query_one_with_reconnect(
            "
            INSERT INTO event_indexers (
                contract_name,
                contract_address,
                chain_id,
                start_block
            ) VALUES ($1, $2, $3, $4)
            RETURNING id;
            ",
            &[
                &contract_name,
                &contract_address,
                &chain_id,
                &start_block,
            ] 
             
        ).await;

        result.map(|row| row.get(0))
    }

    pub async fn update_current_indexing_block(
        indexer_id: i32,
        current_block: i64,
        psql_db: &mut Database,
    ) -> Result<(), PostgresModelError> {
        psql_db.execute_with_reconnect(
            "
            UPDATE event_indexers
            SET current_indexing_block = $1
            WHERE id = $2;
            ",
            &[&current_block, &indexer_id] 
           
        ).await?;

        Ok(())
    }

    pub async fn update_is_synced(
        indexer_id: i32,
        is_synced: bool, 
        psql_db: &mut Database,
    ) -> Result<(), PostgresModelError> {
        psql_db.execute_with_reconnect(
            "
            UPDATE event_indexers
            SET synced = $1
            WHERE id = $2;
            ",
            &[&is_synced,&indexer_id] 
            
        ).await?;

        Ok(())
    }
}

fn decimal_to_u64(input: &Decimal) -> Option<U64> {
    input.to_u128().and_then(|val| {
        if val > u64::MAX as u128 {
            None
        } else {
            Some(U64::from(val as u64))
        }
    })
}
