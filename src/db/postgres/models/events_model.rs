



use ethers::{types::Address, utils::to_checksum};

 
use crate::{db::postgres::postgres_db::Database, event::ContractEvent};

use super::model::PostgresModelError;
 
 
    
   
pub struct EventsModel {
    
    
}


impl EventsModel {  
     
    pub async fn insert_one(
    event: &ContractEvent  ,
  
    psql_db: &Database,
) -> Result<i32, PostgresModelError> {
       
         
         let contract_address = to_checksum(&event.address, None).to_string();
         
         let name = &event.name;
         
         let signature =  serde_json::to_string(  &event.signature ).unwrap();
         
         let args = serde_json::to_string( &event.args ).unwrap();
         
         let data = serde_json::to_string( &event.data ).unwrap() ; 
         
         let transaction_hash =  serde_json::to_string( &event.transaction_hash ).unwrap();
         
         let block_hash =  serde_json::to_string(  &event.block_hash ).unwrap() ;
         
         
         let block_number: &String =  &event.block_number.unwrap().low_u64().to_string();
         
         let log_index: i64 = event.log_index.unwrap().low_u64() as i64;
         
         let transaction_index: i64 = event.transaction_index.unwrap().low_u64() as i64;
          
         
        
        
        let insert_result = psql_db.query_one(
            "
            INSERT INTO events 
            (
            contract_address,
            name,
            signature,
            args,
            data,
            transaction_hash,
            block_number,
            block_hash,
            log_index,
            transaction_index            
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id;
            ",
            &[
                &contract_address, 
                &name,
                &signature,
                &args,
                &data,
                &transaction_hash,
                &block_number,
                &block_hash,
                &log_index  ,
                &transaction_index                 
                ],
        ).await;
    
        match insert_result {
            Ok(row) => Ok(row.get(0)),  // Successfully inserted new row and got its ID.
            Err(e) => {
                eprintln!("Database error: {:?}", e);
                
                Err(  PostgresModelError::Postgres(e) )
                
                
            }
        }
    }
     
    
}