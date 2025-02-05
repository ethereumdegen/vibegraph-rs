



use ethers::{types::{Address, H256}, utils::{to_checksum}};
use rust_decimal::Decimal;

 
use crate::{ event::ContractEvent};

use degen_sql::db::postgres::models::model::PostgresModelError;
use degen_sql::db::postgres::postgres_db::Database;
 
 
use std::str::FromStr;
use ethers::types::{U256,U64};
   
   
pub struct EventsModel {
    
    
}


impl EventsModel {  
     
    pub async fn insert_one(
    event: &ContractEvent  ,
  
    psql_db: &Database,
) -> Result<i32, PostgresModelError> {
       
         
         let contract_address = to_checksum(&event.address, None).to_string();
         
         let name = &event.name;
         
         let signature =  format!( "{:?}", &event.signature )  ; // serde_json::to_string(  &event.signature ).unwrap();
         
         let args = serde_json::to_string( &event.args ).unwrap();
         
         let data = serde_json::to_string( &event.data ).unwrap() ; 
         
         let transaction_hash =  format!( "{:?}", &event.transaction_hash.ok_or_else(|| PostgresModelError::RowParseError)? )  ;
         
         
         let block_hash = format!( "{:?}", &event.block_hash.ok_or_else(|| PostgresModelError::RowParseError)? )  ;
         
         let chain_id = event.chain_id as i64;
         
         
         let block_number_string: &String =  &event.block_number.unwrap().low_u64().to_string();
         let block_number = Decimal::from_str(block_number_string).unwrap();
         
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
            chain_id,
            transaction_hash,
            block_number,
            block_hash,
            log_index,
            transaction_index            
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id;
            ",
            &[
                &contract_address, 
                &name,
                &signature,
                &args,
                &data,
                &chain_id,
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
                eprintln!("Database error: Event {:?}", e);
                
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
            chain_id,
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
                address: Address::from_str ( contract_address ) .map_err(|_e|  PostgresModelError::RowParseError )? ,
                name: row.get("name"),
                signature: H256::from_str(&row.get::<_, String>("signature")).unwrap().into(),
                args: serde_json::from_str(&row.get::<_, String>("args")).unwrap(),
                data: serde_json::from_str(&row.get::<_, String>("data")).unwrap(),
                chain_id:   (row.get::<_, i64>("chain_id")) as u64 ,
                transaction_hash: H256::from_str(&row.get::<_, String>("transaction_hash")).ok(),
                block_number:  U64::from_str(&row.get::<_, Decimal>("block_number").to_string()).ok(),
                block_hash: H256::from_str(&row.get::<_, String>("block_hash")).ok(),
                log_index: Some( (row.get::<_, i64>("log_index")).into()),
                transaction_index: Some( (row.get::<_, i64>("transaction_index")).into()),
                // ... any other fields you might have in the ContractEvent struct
            };
            
            
            Ok( event ) 
            
            
        }, 
        Err(e) => {
            eprintln!("Database error: Recent Event {:?}", e);
            Err(PostgresModelError::Postgres(e))
        }
    }
}
     
     
    
}