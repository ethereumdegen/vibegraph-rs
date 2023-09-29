

 
use std::collections::HashMap;

use ethers::abi::{RawLog, LogParam};
use ethers::providers::{JsonRpcClient, ProviderError};
use tokio::sync::Mutex;
use tokio::time::{sleep, interval, Duration};

use ethers::prelude::{
     abigen, Abigen , 
     Event, Provider, Middleware,Contract};
use ethers::types::{Log, Filter, Address, U256, U64, H256};
use vibegraph_rs::event::{read_contract_events, find_most_recent_event_blocknumber};

use std::sync::Arc;
use vibegraph_rs::db::postgres::models::events_model::EventsModel;
use vibegraph_rs::db::postgres::postgres_db::Database;

use dotenvy::dotenv;
use std::env;

use std::str::FromStr;

use ethers::prelude::Http;
use std::error::Error;
 use tokio::select;

use log::*;

  
 
#[derive(Debug, Clone)]
pub struct ChainState {

    pub most_recent_block_number : Option<U64>,
     
}
  
impl Default for ChainState {
    fn default() -> Self {
        Self{
            most_recent_block_number: None 
        }
    }
}
 
   


#[derive(Debug, Clone)]
pub struct IndexingState {

    pub current_indexing_block : U64,
    pub synced: bool,
    
    pub provider_failure_level: u32

}


impl Default for IndexingState {

    fn default() -> Self {
        Self {
            current_indexing_block: 0.into(),
            synced: false,
            provider_failure_level: 0
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
 
  

//immutable 
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub contract_config: ContractConfig,
    pub indexing_config: IndexingConfig,
}
 
pub struct AppState {
    pub database: Arc<Database>,
 
    pub indexing_state: IndexingState ,
    
}

 
//should happen every 5 seconds 
//is able to mutate app state
async fn collect_events( 
    mut app_state:AppState ,
    app_config: &AppConfig,
    chain_state: Arc<Mutex<ChainState>>
) -> AppState {

    /*
        1. we need to know a start blocknumber. Use indexing config 


    */


    //chug through with this 
    // read_contract_events 

    let rpc_uri = &app_config.indexing_config.rpc_uri;

    let provider = Provider::<Http>::try_from(rpc_uri).unwrap( );
    
    
    let most_recent_block_number = match chain_state.lock().await.most_recent_block_number.clone(){
        
        Some(block_number) => block_number,
        None => {  
            
            //could not read recent block number so we cant continue 
            return app_state
        }
        
    };
    
    
    
         
    let contract_address = Address::from_str(&app_config.contract_config.address)
        .expect("Failed to parse contract address");
        
    let mut block_gap:u32 = match app_state.indexing_state.synced {
        true => {app_config.indexing_config.fine_block_gap }
        false => {app_config.indexing_config.course_block_gap }
    };
    
    
    
    
    if app_state.indexing_state.provider_failure_level > 0  {
        let block_gap_division_factor = std::cmp::min( app_state.indexing_state.provider_failure_level , 8 );
        
        block_gap = std::cmp::min( 1 , block_gap / block_gap_division_factor );        
    }
    
    
    let contract_abi: &ethers::abi::Abi  = &app_config.contract_config.abi; 

  
    let start_block = app_state.indexing_state.current_indexing_block;
    
    
    
    //if we are synced up to 4 blocks from the chain head, skip collection. 
    if start_block > most_recent_block_number - 4 {
        info!( "Fully synced- skipping event collection" );
        return app_state
    }
 
     info!("index starting at {}", start_block);
    
    
    let mut end_block = start_block + std::cmp::max(block_gap - 1, 1);
    
    if end_block >= most_recent_block_number {
        end_block = most_recent_block_number;
        app_state.indexing_state.synced = true;
    }

    let event_logs = match read_contract_events(
        contract_address,
        contract_abi,
        start_block,
        end_block,
        provider
    ).await {
        Ok( evts ) => evts,
        Err(e) => { 
                
           
            //we increase the failure level which will shrink the block gap to ease the load on the provider in case that was the issue
            app_state.indexing_state.provider_failure_level += 1 ; 
                            
              //max failure level is 8 
            app_state.indexing_state.provider_failure_level = std::cmp::min( app_state.indexing_state.provider_failure_level , 8 );
              
            return app_state
        }       
    };
        
        
    //on success we reduce the failure level 
    if app_state.indexing_state.provider_failure_level >= 1 {
          app_state.indexing_state.provider_failure_level -= 1 ; 
    }
  
    
    
    for event_log in event_logs {
        
          info!("decoded event log {:?}", event_log);
          
          let psql_db = &app_state.database;
          
          EventsModel::insert_one(&event_log, psql_db).await;
        
    }
    

    //progress the current indexing block
    app_state.indexing_state.current_indexing_block = end_block + 1; 



    app_state
} 



 
 

async fn initialize(
    mut app_state: AppState, 
    app_config: &AppConfig ,
    chain_state: &Arc<Mutex<ChainState>>
    
    ) -> AppState {
 
    
     
    //initialize state 

      
    let app_config_arc = Arc::new( &app_config );
    
    
    info!("Initializing state");
    
    
    
    
    let contract_address = Address::from_str( &app_config.contract_config.address ).unwrap();
    
    let most_recent_event_blocknumber = find_most_recent_event_blocknumber(
        contract_address, 
        &app_state.database
    ).await;
    
    app_state.indexing_state.current_indexing_block = match most_recent_event_blocknumber {
        Some(recent_blocknumber) => recent_blocknumber, //start from recent event .. where we left off  
        None => app_config.contract_config.start_block.clone() //start from beginning 
    };
    
   
 
    let mut initialize_loop_interval = interval(Duration::from_secs(2));
    loop {
        
        let collect_most_recent_block =  collect_blockchain_data(
             &Arc::clone(&app_config_arc),
             
             ).await;
             
        if let Ok(block_number) = collect_most_recent_block {
            
            chain_state.lock().await.most_recent_block_number = Some( block_number );
            break;//break the init loop 
        }
        
        initialize_loop_interval.tick().await; //try again after delay 
    }
    
    info!("Initialization complete");
    

   
    app_state 

}

 
   

async fn start(
    mut app_state: AppState, 
    app_config: &AppConfig ,
    chain_state: &Arc<Mutex<ChainState>>
    
    ){
 
    
     
    let app_config_arc = Arc::new( &app_config );

    let mut collect_events_interval = interval(Duration::from_secs(5));
    
    let mut collect_blockchain_data_interval = interval( Duration::from_secs(40) );
  
     loop {
        select! {
            _ = collect_events_interval.tick() => {
                app_state = collect_events(
                    app_state,
                     &Arc::clone(&app_config_arc), 
                     Arc::clone(&chain_state)).await;
            }
            _ = collect_blockchain_data_interval.tick() => {
                if let Ok(block_number) = collect_blockchain_data(
                     &Arc::clone(&app_config_arc), 
                     ).await{
                           chain_state.lock().await.most_recent_block_number = Some( block_number );
                     }
            }
        }
    }
    

}




async fn collect_blockchain_data(  
     app_config: &AppConfig, 
   //  chain_state: Arc<Mutex<ChainState>>
      ) -> Result< U64, ProviderError> {
    
     let rpc_uri = &app_config.indexing_config.rpc_uri;

     let provider = Provider::<Http>::try_from(rpc_uri).unwrap( );
    
     let block_number = provider.get_block_number().await?;
     info!("Current block number: {}", block_number);
        
   
    
     Ok(block_number)
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
         
        indexing_state
       
    };
    
     let  app_config = AppConfig {
        
        indexing_config,
        contract_config: alt_contract_config,
        
       
    };
   
    let chain_state = Arc::new(Mutex::new(ChainState::default()));
    
    
    
    app_state = initialize(app_state, &app_config, &Arc::clone(&chain_state)).await;

    start(app_state, &app_config, &Arc::clone(&chain_state)).await;
    
    
    
}
