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
use degen_sql::db::postgres::postgres_db::{Database };



#[derive(Clone,Debug)]
pub struct EventIndexer {
    //pub id: u64,
    pub name: String, 
    pub contract_name: String,
    pub contract_address: Address,
    pub chain_id: u64,
    pub start_block: u64,
    pub current_indexing_block: Option<u64>,
    pub synced: bool,
  //  pub created_at: DateTime<Utc>,
}

impl EventIndexer {
	pub fn new (
        name: String, 
		contract_name: String, 
		contract_address: Address,
		chain_id: u64, 
		start_block: u64,


	) -> Self  {

		Self {

            name, 
			contract_name,   
			contract_address,
			chain_id,
			start_block,
			current_indexing_block: None,
			synced: false 

		}




	}


    pub fn from_row(row: &Row) -> Result<Self, PostgresModelError>{

          let contract_address = Address::from_str(&row.get::<_, String>("contract_address"))
                .map_err(|e| PostgresModelError::RowParseError(format!("Invalid contract address: {:?}", e).into()))?;



        Ok( Self{ 
        	     name: row.get("name"), 
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

        table_name: String, 
	    offset_indexer_id: Option<i32>,
	    psql_db: &  Database,
	) -> Result< ( i32 , EventIndexer ), PostgresModelError> {
	    
	     

        let row = match offset_indexer_id {


        	Some(index) => {


        		let index = index as i32; 


        		   let query =format!("
			            SELECT id, name, contract_name, contract_address, chain_id, start_block, current_indexing_block, synced, created_at
			            FROM  {}
			            WHERE id > $1
			            ORDER BY id ASC
			            LIMIT 1;
			        " ,table_name);

			        psql_db.query_one(&query, &[   &index]).await? 

        	},

        	None => {

        		   let query = format!("
			            SELECT id, name, contract_name, contract_address, chain_id, start_block, current_indexing_block, synced, created_at
			            FROM  {}
			           
			            ORDER BY id ASC
			            LIMIT 1;
			        ",table_name);

			        psql_db.query_one(&query, &[  ]).await?



        	}



        };
 	

        let indexer = EventIndexer::from_row(&row)? ;


        let id = row.get::<_, i32>("id")    ; 


 		Ok(  ( id , indexer ) )
	    
	}
	     
	     


	 pub async fn insert_one(
         table_name: String,  
        indexer: &EventIndexer,
        psql_db: &  Database,
    ) -> Result<u64, PostgresModelError> { // Return type changed to u64 to match the id type
        let name = &indexer.name;
        let contract_address = format!( "{:?}" , indexer.contract_address );
        let contract_name = &indexer.contract_name;

        let chain_id = indexer.chain_id as i64; // Casting to i64 as PostgreSQL does not support u64 natively
        let start_block = indexer.start_block as i64; // Same as above


         let query = format!(
            "
            INSERT INTO {} (
                name, 
                contract_name,
                contract_address,
                chain_id,
                start_block
            ) VALUES ($1, $2, $3, $4, $5) RETURNING id;
            ",
            table_name // Table name injected safely as it's a trusted input
        );

        let result = psql_db.execute(
           &query,
            &[ 
            
               &name, 
                &contract_name,
                &contract_address,
                &chain_id,
                &start_block,
                 
            ],
        ).await;

        Ok(result.map(|row| row     )?) // Extracting the first column which is 'id'
    }

    pub async fn update_current_indexing_block(
         table_name: String, 
        indexer_id: i32,
        current_block: u64,
        psql_db: &  Database,
    ) -> Result<(), PostgresModelError> {


    	let current_block = current_block as i64; 

    	let query = format!( 

              "
            UPDATE  {}
            SET current_indexing_block = $1
            WHERE id = $2;
            ", table_name
            );
        psql_db.execute (
          &query,
            &[  &current_block, &indexer_id] 
           
        ).await?;

        Ok(())
    }

    pub async fn update_is_synced(
         table_name: String, 
        indexer_id: i32,
        is_synced: bool, 
        psql_db: &  Database,
    ) -> Result<(), PostgresModelError> {

        let query = format!( 

              "
             UPDATE  {}
            SET synced = $2
            WHERE id = $3;
            ", table_name
            );

        psql_db.execute (
          &query,
            &[  &is_synced,&indexer_id] 
            
        ).await?;

        Ok(())
    }
} 