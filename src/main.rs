

 
use std::collections::HashMap;

use ethers::abi::{RawLog, LogParam};
use ethers::providers::{JsonRpcClient, ProviderError};
use tokio::time::{sleep, interval, Duration};

use ethers::prelude::{
     abigen, Abigen , 
     Event, Provider, Middleware,Contract};
use ethers::types::{Log, Filter, Address, U256, U64, H256};
use vibegraph_rs::event::read_contract_events;

use std::sync::Arc;
use vibegraph_rs::db::postgres::models::events_model::EventsModel;
use vibegraph_rs::db::postgres::postgres_db::Database;

use dotenvy::dotenv;
use std::env;

use std::str::FromStr;

use ethers::prelude::Http;
use std::error::Error;
 

use log::*;

  
 
  
 
   


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
 
     info!("index starting at {}", start_block);
    
    
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
    
    
    for event_log in event_logs {
        
          info!("decoded event log {:?}", event_log);
          
          let psql_db = &app_state.database;
          
          EventsModel::insert_one(&event_log, psql_db).await;
        
    }
    

    //progress the current indexing block
    app_state.indexing_state.current_indexing_block = end_block + 1; 



    app_state
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
    env_logger::init();
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
        abi:  serde_json::from_str( abi_string ).unwrap(),   
        start_block: 4382418.into(),
        name: "payspec" .to_string()
    };

    let abi_string_alt = include_str!("../abi/artblox.abi.json");

    let alt_contract_config = ContractConfig {
        address: "0x4590383ae832ebdfb262d750ee81361e690cfc9c".to_string(),
        abi:  serde_json::from_str( abi_string_alt ).unwrap(),   
        start_block: 4182418.into(),
        name: "artblox" .to_string()
    };

    let indexing_state = IndexingState::default();
  

    //attach database 
    let database = Arc::new(
        Database::connect().await.unwrap()
    ); 

    let mut app_state = AppState {
        database: Arc::clone(&database),
        indexing_config,
        contract_config: alt_contract_config,
        indexing_state
    };
   

    start(app_state).await;
}
