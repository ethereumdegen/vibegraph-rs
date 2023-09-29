

 



use ethers::providers::{ProviderError};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use ethers::prelude::{
     Provider, Middleware};
use ethers::types::{Address, U64};
use vibegraph::{IndexingConfig, ContractConfig, AppConfig, Vibegraph};
use vibegraph::event::{read_contract_events, find_most_recent_event_blocknumber};

use std::sync::Arc;
use vibegraph::db::postgres::models::events_model::EventsModel;
use vibegraph::db::postgres::postgres_db::Database;

use dotenvy::dotenv;

use serde::Deserialize;
use serde_json;


use std::str::FromStr;

use ethers::prelude::Http;

 use tokio::select;

use log::*;

   
  

/*

cargo run config/payspec_config.json

*/

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
    
    let path = std::env::args().nth(1).expect("Please provide a file path");
    let config_content = std::fs::read_to_string(path).expect("Could not read config");
    
    let contract_config: ContractConfig = serde_json::from_str(&config_content).expect("Could not parse config");
    
    
     let  app_config = AppConfig {
        
        indexing_config,
        contract_config 
       
    };
    
    
    
    Vibegraph::init( &app_config ).await;
    
    
}

 