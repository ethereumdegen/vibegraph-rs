

 
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
mod db; 


/*
mod abis {
    ethers::contract::abigen!(
        PayspecAbi,
        "abi/payspec.abi.json",
        event_derives (serde::Deserialize, serde::Serialize);
    );
}

*/



#[derive(Debug, Clone)]
pub struct IndexingState {

    pub current_indexing_block : U64,
    pub synced: bool 

}


impl Default for IndexingState {

    fn default() -> Self {
        Self {
            current_indexing_block: 0.into(),
            synced: false 
        }

    }

}


#[derive(Debug, Clone)]
pub struct IndexingConfig {
    pub rpc_uri: String,
    pub index_rate: u32,
    pub update_block_number_rate: u32,
    pub course_block_gap: u32,
    pub fine_block_gap: u32,
    pub safe_event_count: u32,

    //pub subscribe: Option<bool>,


    //pub log_level: Option<String>,
    //pub contracts: Vec<ContractConfig>,
 
   


    //pub custom_indexers: Vec<CustomIndexer>,
    // Callback functions can be represented using Boxed closures in Rust.
    // The exact type depends on the function signature, which isn't provided.
    // Here's a generic example.
    //pub on_index_callback: Option<Box<dyn Fn()>>,
}



#[derive(Debug, Clone)]
pub struct ContractConfig {
    pub address: String,
    pub start_block: U64,
    pub name: String,  
    pub abi: ethers::abi::Abi 
}

 

pub const DEFAULT_FINE_BLOCK_GAP: u32 = 20;
pub const DEFAULT_COURSE_BLOCK_GAP: u32 = 8000;
 

/*
#[derive(Debug, Clone)]
pub struct ContractEvent {
    pub name: String,
    pub signature: String,
    pub args: HashMap<String, String>,  // Assuming args can be represented as key-value pairs
    pub address: String,
    pub data: String,
    pub transaction_hash: String,
    pub block_number: u64,
    pub block_hash: String,
    pub log_index: u32,
    pub transaction_index: u64,
}
  */


/*
#[derive(Debug, Clone)]
pub struct ContractEventRaw {
    pub block_number: U256,
    pub block_hash: String,
    pub transaction_index: U256,
    pub address: String,
    pub data: String,
    pub topics: Vec<String>,
    pub transaction_hash: String,
    pub log_index: U256,
}*/

#[derive(Debug,Clone)]
struct ContractEventRaw {
     block_number: Option<U64>,
    block_hash: Option< H256 >,
     transaction_index: Option<U64>,
     transaction_hash: Option< H256 > ,
         data: Vec<u8>,
    topics: Vec<H256> , 
  
    address: Address,  
    log_index: Option < U256 > ,
   
}

impl ContractEventRaw {
    
    pub fn from_log(evt:Log) -> Self {
         Self {
                address: evt.address,
                topics: evt.topics,
                data:  evt.data.0.to_vec(),
                transaction_hash: evt.transaction_hash,
                transaction_index: evt.transaction_index ,
                block_number: evt.block_number,
                block_hash: evt.block_hash,
                log_index: evt.log_index , 
            } 
            
    }
    
}

#[derive(Debug)]
struct ContractEvent {
    name: String,
    signature: H256,
    args: Vec< LogParam >, // Adjust the data structure as per your needs
    address: Address,
    data: Vec<u8>,
    transaction_hash: Option< H256 > ,
    block_number: Option<U64>,
    block_hash: Option< H256 >,
    log_index: Option < U256 > ,
    transaction_index: Option<U64>,
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
 



 
pub struct AppState {
    pub database: Arc<Database>,
    pub contract_config: ContractConfig,
    pub indexing_config: IndexingConfig ,
    pub indexing_state: IndexingState  
}

 
//should happen every 5 seconds 
//is able to mutate app state
async fn collect_events( 
    mut app_state:AppState 
) -> AppState {

    /*
        1. we need to know a start blocknumber. Use indexing config 


    */


    //chug through with this 
    // read_contract_events 

    let rpc_uri = &app_state.indexing_config.rpc_uri;

    let provider = Provider::<Http>::try_from(rpc_uri).unwrap( );
    
         
   let contract_address = Address::from_str(&app_state.contract_config.address)
        .expect("Failed to parse contract address");
        
    let block_gap = match app_state.indexing_state.synced {
        true => {app_state.indexing_config.fine_block_gap }
        false => {app_state.indexing_config.course_block_gap }
    };
    
    let contract_abi: &ethers::abi::Abi  = &app_state.contract_config.abi; 

  
    let start_block = app_state.indexing_state.current_indexing_block;
 
      println!("index starting at {}", start_block );
    
    
    let end_block = start_block + std::cmp::max(block_gap - 1, 1);

    let event_logs = match read_contract_events(
        contract_address,
        contract_abi,
        start_block,
        end_block,
        provider
    ).await {
        Ok( evts ) => evts,
        Err(e) => { 
                
            //we may need to try reducing the block gap   here !    
            //since we got a provider error   
              
            return app_state
        }       
    };

    //progress the current indexing block
    app_state.indexing_state.current_indexing_block = end_block + 1; 



    app_state
} 




async fn read_contract_events<M:  JsonRpcClient>( 
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



fn try_identify_event_for_log(
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



async fn start(mut app_state: AppState){



    //initialize state 

    app_state.indexing_state.current_indexing_block = 
        app_state.contract_config.start_block;





   

    let mut interval = interval(Duration::from_secs(5));


    loop {
 

        app_state = collect_events( app_state ).await;


        interval.tick().await;  
      

    }

}




#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let rpc_uri =  std::env::var("RPC_URL")
        .expect("RPC_URL must be set");

    //mocked for now -- move to json and serde it ? 
    let indexing_config = IndexingConfig {
         rpc_uri,
         index_rate: 4000, //ms
         update_block_number_rate: 5000,  //ms
         course_block_gap: 2000,
         fine_block_gap: 100,
         safe_event_count: 400,

    };
     
    let abi_string = include_str!("../abi/payspec.abi.json");
    
    let contract_config = ContractConfig {
        address: "0xdC726D36a2f1864D592fF8d420710cd2C3D350aa".to_string(),
        abi:  serde_json::from_str( abi_string ).unwrap(),  //Abigen::new("Payspec", "abi/payspec.abi.json").unwrap(),
        start_block: 4382418.into(),
        name: "payspec" .to_string()
    };

    let indexing_state = IndexingState::default();
  

    //attach database 
    let database = Arc::new(
        Database::connect().await.unwrap()
    ); 

    let mut app_state = AppState {
        database: Arc::clone(&database),
        indexing_config,
        contract_config,
        indexing_state
    };
   

    start(app_state).await;
}
