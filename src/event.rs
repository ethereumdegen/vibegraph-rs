
use std::collections::HashMap;

use ethers::abi::{RawLog, LogParam};
use ethers::providers::{JsonRpcClient, ProviderError};
use tokio::time::{sleep, interval, Duration};

use ethers::prelude::{
     abigen, Abigen , 
     Event, Provider, Middleware,Contract};
use ethers::types::{Log, Filter, Address, U256, U64, H256};

use std::sync::Arc;
use crate::db::postgres::postgres_db::Database;

use dotenvy::dotenv;
use std::env;

use std::str::FromStr;

use ethers::prelude::Http;
use std::error::Error;
  

use log::*;

  


#[derive(Debug)]
pub struct ContractEvent {
    pub name: String,
    pub signature: H256,
    pub args: Vec< LogParam >,  
    pub address: Address,
    pub data: Vec<u8>,
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
            transaction_hash: evt.transaction_hash,
            transaction_index: evt.transaction_index ,
            block_number: evt.block_number,
            block_hash: evt.block_hash,
            log_index: evt.log_index , 
            
        }
    } 
    
}
 



pub async fn read_contract_events<M:  JsonRpcClient>( 
    contract_address: Address,
    contract_abi:  &ethers::abi::Abi,
    start_block: U64,
    end_block: U64,
    provider: Provider<M>,

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
                ContractEvent::new(name, signature, args, evt)
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
