



use rust_decimal::prelude::ToPrimitive;
use log::info;
use ethers::{types::{Address, H256}, utils::{to_checksum}};
use rust_decimal::Decimal;
use tokio::time::timeout;

 
use crate::{ event::ContractEvent};

use degen_sql::db::postgres::{models::model::PostgresModelError  };
use degen_sql::db::postgres::postgres_db::Database;
 
 
use std::{str::FromStr, time::Duration};
use ethers::types::{U256,U64};
   
   
pub struct EventsModel {
    
    
}


impl EventsModel {  
     
    pub async fn insert_one(
    event: &ContractEvent  ,
  
    psql_db: &  Database,
) -> Result<i32, PostgresModelError> {
       
         
         let contract_address = to_checksum(&event.address, None).to_string();
         
         let name = &event.name;
         
         let signature =  format!( "{:?}", &event.signature )  ; // serde_json::to_string(  &event.signature ).unwrap();
         
         let args = serde_json::to_string( &event.args ).unwrap();
         
         let data = serde_json::to_string( &event.data ).unwrap() ;     

         let transaction_hash = format!(
            "{:?}",
            &event.transaction_hash.ok_or_else(|| PostgresModelError::RowParseError( Some("Missing transaction hash".to_string())  ))?
        );

         
         
         let block_hash = format!(
                "{:?}",
                &event.block_hash.ok_or_else(|| PostgresModelError::RowParseError( Some("Missing block hash".to_string())   ))?
           );
         
         
         
         let chain_id = event.chain_id as i64;
         
         
         let block_number_string: &String =  &event.block_number.unwrap().low_u64().to_string();
         let block_number = Decimal::from_str(block_number_string).unwrap();
         
         let log_index: i64 = event.log_index.unwrap().low_u64() as i64;
         
         let transaction_index: i64 = event.transaction_index.unwrap().low_u64() as i64;
          
         
            


        // Set a timeout (e.g., 5 seconds)
        let insert_result =  
            psql_db.query_one(
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
                    &log_index,
                    &transaction_index
                ] 
            
        ).await;

          Ok(insert_result.map(|r| r.get::<_,i32>("id"))?) 
         
    }
     
     
pub async fn find_most_recent_event(
    contract_address: Address,
    psql_db: &  Database,
) -> Result< ContractEvent, PostgresModelError> {
    
    
    let parsed_contract_address = to_checksum(&contract_address, None).to_string();
         
    
    let row = psql_db.query_one (
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
        &[&parsed_contract_address] 
    ).await;

    match row {
        Ok(row) => {
            
            
            
 
            let event = ContractEvent ::from_row(&row) ?;
            
            Ok( event ) 
            
            
        }, 
        Err(e) => {
            eprintln!("Database error: Recent Event {:?}", e);
            Err( e .into() ) 
        }
    }
}
     
     
    
}


fn decimal_to_u64 (input: &Decimal) -> Option< U64  > {


      // Scale the decimal value
    let scaled_decimal = input  ;

    // Ensure the value can be represented as a u64
    let u128_value = scaled_decimal
        .to_u128()
        .expect("Failed to convert Decimal to u64");

     // Ensure it fits in a u64
    if u128_value > u64::MAX as u128 {
      //  panic!("Value exceeds u64 range");
      return None; 
    }

    Some(  U64::from(u128_value as u64) )
}