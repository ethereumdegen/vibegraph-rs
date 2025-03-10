


 use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::PostgresModelError;
use log::info;
use ethers::abi::{LogParam};
use ethers::providers::{JsonRpcClient, ProviderError};


use ethers::prelude::{
     Provider, Middleware,Contract};
use ethers::types::{Log, Filter, Address, U256, U64, H256};
use tokio_postgres::Row;

use std::sync::Arc;
use crate::db::postgres::models::events_model::EventsModel;
use degen_sql::db::postgres::postgres_db::Database;








  



  


#[derive(Debug)]
pub struct ContractEvent {
    pub name: String,
    pub signature: H256,
    pub args: Vec< LogParam >,  
    pub address: Address,
    pub data: Vec<u8>,
    pub chain_id: u64,
    pub transaction_hash: Option< H256 > ,
    pub block_number: Option<U64>,
    pub block_hash: Option< H256 >,
    pub log_index: Option < U256 > ,
    pub transaction_index: Option<U64>,
}

impl ContractEvent {
   
    pub fn new( 
        
        name: String,
        signature:H256, 
        chain_id: u64,
        args: Vec<LogParam>,
        evt:  Log,
            
    ) ->  Self {
        Self {
            name,
            signature,
            args,
             
            address: evt.address,
               // topics: evt.topics,
            data:  evt.data.0.to_vec(),
            chain_id,
            transaction_hash: evt.transaction_hash,
            transaction_index: evt.transaction_index ,
            block_number: evt.block_number,
            block_hash: evt.block_hash,
            log_index: evt.log_index , 
            
        }
    } 

    pub fn from_row(row: &Row) -> Result<Self, PostgresModelError>{

          let contract_address = Address::from_str(&row.get::<_, String>("contract_address"))
                .map_err(|e| PostgresModelError::RowParseError(format!("Invalid contract address: {:?}", e).into()))?;



        Ok( Self{ 
                address:  contract_address  ,
                name: row.get("name"),
                signature: H256::from_str(&row.get::<_, String>("signature")).unwrap().into(),
                args: serde_json::from_str(&row.get::<_, String>("args")).unwrap(),
                data: serde_json::from_str(&row.get::<_, String>("data")).unwrap(),
                chain_id:   (row.get::<_, i64>("chain_id")) as u64 ,
                transaction_hash: H256::from_str(&row.get::<_, String>("transaction_hash")).ok(),
                block_number:   decimal_to_u64(  &row.get::<_, Decimal>("block_number") ) ,
                block_hash: H256::from_str(&row.get::<_, String>("block_hash")).ok(),
                log_index: Some( (row.get::<_, i64>("log_index")).into()),
                transaction_index: Some( (row.get::<_, i64>("transaction_index")).into()),
            })


    }
    
}
 



pub async fn read_contract_events<M:  JsonRpcClient>( 
    contract_address: Address,
    contract_abi:  &ethers::abi::Abi,
    start_block: U64,
    end_block: U64,
    provider: Provider<M>,
    chain_id: u64,

 )-> Result< Vec<ContractEvent >, ProviderError>  {


   // let provider = Provider::<Http>::try_from(HTTP_URL)?;
    let client = Arc::new(provider);
    
    //https://www.gakonst.com/ethers-rs/events/logs-and-filtering.html
    let filter = Filter::new()
    .address(vec![contract_address])
    .from_block(start_block )
    .to_block(end_block ) ;
    
    //https://docs.rs/ethers-contract/latest/ethers_contract/
    
    //https://github.com/gakonst/ethers-rs/issues/1810
    //https://github.com/ethers-io/ethers.js/issues/179
    
    let raw_events: Vec<Log> = client.get_logs(&filter).await?;

  //   https://github.com/gakonst/ethers-rs/issues/2541
    
    let contract = Contract::new(
        contract_address, contract_abi.clone(), Arc::new(client)
        ) ;
        
   
        
    let event_logs = raw_events
        .into_iter()
        . filter_map(  |evt| { 
            
             try_identify_event_for_log(evt.clone(), &contract.abi())
            .map(
                |(name, signature, args)| 
                ContractEvent::new(name, signature, chain_id, args, evt)
                )
             
            
        }).collect();
     
      
    

    Ok( event_logs )

 

}




pub fn try_identify_event_for_log(
    evt: Log, 
    contract_abi:  &ethers::abi::Abi
) -> Option<(String, H256, Vec<LogParam>) > {
     let contract_events = contract_abi.events();
      
      for abi_event in contract_events { 
                let abi_event_topic = abi_event.signature();  
                    
                    if let Some(evt_topic) =  evt.topics.first() {
                        if  abi_event_topic == *evt_topic  {
                            let event_name = abi_event.name.clone();
                            let full_log =  abi_event.parse_log( evt.into() ).unwrap(); 
                            let full_log_params = full_log.params; 
                                
                                //name , signature, args 
                             return Some((event_name,abi_event_topic, full_log_params)) 
                        }
                        
                        
                    }  
            }
    None 
}

/*
pub async fn find_most_recent_event_blocknumber( 
    contract_address: Address,
    psql_db: &mut Database
) -> Option<U64> {
    
    
    let most_recent_event_for_contract_address = 
        EventsModel::find_most_recent_event(contract_address,psql_db).await.ok();
            
        info!("most recent event {:?}", most_recent_event_for_contract_address);
        
        //flat map 
      most_recent_event_for_contract_address.and_then(|event|  event.block_number )
    
      
}*/

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