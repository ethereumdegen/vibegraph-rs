



use ethers::{types::Address, utils::{to_checksum, hex::FromHexError}};

 
use crate::{db::postgres::postgres_db::Database, event::ContractEvent};

use super::model::PostgresModelError;
 
 
use std::str::FromStr;
   
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
     
     
pub async fn find_most_recent_event(
    contract_address: Address,
    psql_db: &Database,
) -> Result< ContractEvent, PostgresModelError> {
    
    
    let parsed_contract_address = to_checksum(&contract_address, None).to_string();
         
    
    let row = psql_db.query_one(
        "
        SELECT 
            contract_address,
            name,
            signature,
            args,
            data,
            transaction_hash,
            block_number,
            block_hash,
            log_index,
            transaction_index,
            created_at
        FROM events
        WHERE (contract_address) = ($1)
        ORDER BY created_at DESC
        LIMIT 1;
        ",
        &[&parsed_contract_address],
    ).await;

    match row {
        Ok(row) => {
            
            
            let contract_address =  &row.get::<_, String>("contract_address"); 
            
            
            let event = ContractEvent {
                address: Address::from_str ( contract_address ) .map_err(|e|  PostgresModelError::AddressParseError )? ,
                name: row.get("name"),
                signature: serde_json::from_str(&row.get::<_, String>("signature")).unwrap(),
                args: serde_json::from_str(&row.get::<_, String>("args")).unwrap(),
                data: serde_json::from_str(&row.get::<_, String>("data")).unwrap(),
                transaction_hash: serde_json::from_str(&row.get::<_, String>("transaction_hash")).unwrap(),
                block_number: Some(u64::from_str(&row.get::<_, String>("block_number")).unwrap().into()),
                block_hash: serde_json::from_str(&row.get::<_, String>("block_hash")).unwrap(),
                log_index: Some( (row.get::<_, i64>("log_index")).into()),
                transaction_index: Some( (row.get::<_, i64>("transaction_index")).into()),
                // ... any other fields you might have in the ContractEvent struct
            };
            
            
            Ok( event ) 
            
            
        }, 
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(PostgresModelError::Postgres(e))
        }
    }
}
     
     
    
}