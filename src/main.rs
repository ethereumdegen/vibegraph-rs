

 
use std::collections::HashMap;

use tokio::time::{sleep, interval, Duration};

use ethers::prelude::{Event, Provider, Middleware,Contract};
use ethers::types::{Log, Filter, Address, U256};

use std::sync::Arc;
use crate::db::postgres::postgres_db::Database;

use dotenvy::dotenv;
use std::env;

use ethers::prelude::Http;
use std::error::Error;
mod db; 





#[derive(Debug, Clone)]
pub struct IndexingState {

    pub current_indexing_block : U256,
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
    pub start_block: U256,
    pub name: String,   
}

 

pub const DEFAULT_FINE_BLOCK_GAP: u32 = 20;
pub const DEFAULT_COURSE_BLOCK_GAP: u32 = 8000;

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
}


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


#[derive(Debug)]
struct ContractEvent {
    name: String,
    signature: String,
    args: Vec<(String, String)>, // Adjust the data structure as per your needs
    address: Address,
    data: Vec<u8>,
    transaction_hash: Vec<u8>,
    block_number: U256,
    block_hash: Vec<u8>,
    log_index: U256,
    transaction_index: U256,
}

#[derive(Debug)]
struct ContractEventsResult {
    contract_address: Address,
    from_block: U256,
    to_block: U256,
    events: Vec<ContractEvent>,
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

    let rpc_uri = app_state.indexing_config.rpc_uri;

    let provider = Provider::<Http>::try_from(rpc_uri).unwrap( );

    let contract_address = app_state.contract_config.address;
    
    let block_gap = match app_state.indexing_state.synced {
        true => {app_state.indexing_config.course_block_gap }
        false => {app_state.indexing_config.fine_block_gap }
    };


    let start_block = app_state.indexing_state.current_indexing_block;
 
    let end_block = start_block + std::cmp::max(block_gap - 1, 1);

    let contract_event_result = read_contract_events(
        contract_address,
        contract_abi,
        start_block,
        end_block,
        provider
    ).await;


    //progress the current indexing block
    app_state.indexing_state.current_indexing_block = end_block + 1; 



    app_state
} 




async fn read_contract_events<M: Middleware>( 
    contract_address: Address,
    contract_abi: ethers::abi::Abi,
    start_block: U256,
    end_block: U256,
    provider: Provider<M>,

 )-> Result<ContractEventsResult, Box<dyn Error>>  {


   // let provider = Provider::<Http>::try_from(HTTP_URL)?;
    let client = Arc::new(provider);
    
    //https://www.gakonst.com/ethers-rs/events/logs-and-filtering.html
    let filter = Filter::new()
    .address(vec![contract_address])
    .from_block(start_block)
    .to_block(end_block) ;

    let raw_events: Vec<Log> = client.get_logs(&filter).await?;


    let decoded_events: Vec<ContractEvent> = raw_events
        .into_iter()
        .filter_map(|evt| {
            let decode_result = contract_abi.parse_log(evt.clone()).ok()?;

            Some(ContractEvent {
                name: decode_result.name,
                signature: decode_result.signature,
                args: decode_result.args.iter().map(|(k, v)| (k.clone(), v.to_string())).collect(), 
                address: evt.address,
                data: evt.data.0.to_vec(),
                transaction_hash: evt.transaction_hash,
                block_number: evt.block_number,
                block_hash: evt.block_hash,
                log_index: evt.log_index.unwrap_or(U256::zero()),
                transaction_index: evt.transaction_index.unwrap_or(U256::zero()),
            })
        })
        .collect();

    Ok(ContractEventsResult {
        contract_address,
        from_block: start_block,
        to_block: end_block,
        events: decoded_events,
    })


/*
    println!("{} pools found!", logs.iter().len());
    for log in logs.iter() {
        let token0 = Address::from(log.topics[1]);
        let token1 = Address::from(log.topics[2]);
        let fee_tier = U256::from_big_endian(&log.topics[3].as_bytes()[29..32]);
        let tick_spacing = U256::from_big_endian(&log.data[29..32]);
        let pool = Address::from(&log.data[44..64].try_into()?);
        println!(
            "pool = {pool}, token0 = {token0}, token1 = {token1}, fee = {fee_tier}, spacing = {tick_spacing}"
        );
    }*/



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

    let contract_config = ContractConfig {
        address: "0xdC726D36a2f1864D592fF8d420710cd2C3D350aa".to_string(),
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
   

    start(app_state);
}
