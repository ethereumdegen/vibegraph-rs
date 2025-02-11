



use std::collections::HashMap;
use degen_sql::db::postgres::models::model::PostgresModelError;
use ethers::providers::{ProviderError};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use ethers::prelude::{
     Provider, Middleware};
use ethers::types::{Address, U64, U256};
use event::{read_contract_events, find_most_recent_event_blocknumber};

use std::sync::Arc;
use db::postgres::models::events_model::EventsModel;
use degen_sql::db::postgres::postgres_db::{Database,DatabaseCredentials};



use serde::Deserialize;



use std::str::FromStr;

use ethers::prelude::Http;

 use tokio::select;

use log::*;




/*


 


*/
 

pub mod db;

pub mod event;


 


pub struct Vibegraph {
  
}



impl Vibegraph {
    
    
    
 ///Used to externally start vibegraph 
pub async fn init (
   
    app_config: &AppConfig  
    
){
    
        let indexing_state = IndexingState::default();

        let database_credentials = app_config.db_conn_url.clone()  ;
        

         // Attach database with proper error handling
        let database = match Database::connect(database_credentials, None).await {
            Ok(db) => Arc::new(Mutex::new(db)),  // Wrap in Arc<Mutex<T>> properly
            Err(e) => {
                eprintln!("Failed to connect to database: {}", e);
                return;
            }
        };

        // Proper struct initialization
        let mut app_state = AppState {
            database: Arc::clone(&database), // Clone Arc correctly
            indexing_state,
        };

    /*
        //attach database 
        let database = Arc::new(
            Database::connect(database_credentials,None).await.unwrap()
        ); 
    
        let mut app_state = AppState {
            database: Arc::clone( Mutex::new( database )),            
            indexing_state        
        };
        */
        
        let chain_state = Arc::new(Mutex::new(ChainState::default()));
        
        
        app_state = initialize(app_state, &app_config, &Arc::clone(&chain_state)).await;
    
        start(app_state, &app_config, &Arc::clone(&chain_state)).await;
        
        
        
    }
    
}


  
    
      
        
          
            
                
 
#[derive(Debug, Clone)]
pub struct ChainState {

    pub most_recent_block_number : Option<U64>,
    pub chain_id : Option<U256> 
     
}
  
impl Default for ChainState {
    fn default() -> Self {
        Self{
            most_recent_block_number: None,
            chain_id: None  
        }
    }
}
 
   

// moving this to db ! 
#[derive(Debug, Clone)]
pub struct IndexingState {

   // pub current_indexing_block : U64,
  //  pub synced: bool,

   pub current_indexer_id: Option<U64>,
    
    pub provider_failure_level: u32

}


impl Default for IndexingState {

    fn default() -> Self {
        Self {
          //  current_indexing_block: 0.into(),
          //  synced: false,

            current_indexer_id : None, 

            provider_failure_level: 0
        }

    }

}


#[derive(Debug, Clone)]
pub struct IndexingConfig {
  //  pub rpc_uri: String,
    pub index_rate: u64,
    pub update_block_number_rate: u64,
    pub course_block_gap: u32,
    pub fine_block_gap: u32,
    pub safe_event_count: u32,

    
}


// moving this to db !! 
#[derive(Debug, Clone, Deserialize)]
pub struct ContractConfig {
    pub contract_address: String,
    pub chain_id: u64 , 
    pub start_block: u64,
    pub name: String,  
  //  pub abi: ethers::abi::Abi 
}

  
  

//immutable 
#[derive(Debug, Clone)]
pub struct AppConfig {
 //   pub contract_config_map: HashMap<String, ContractConfig >,  //store in db 


    pub rpc_uri_map: HashMap<u64, String > ,
    pub indexing_config: IndexingConfig,
    pub contract_abi_map: HashMap<String, ethers::abi::Abi>,

    pub db_conn_url : String , 
  //  pub database_credentials: DatabaseCredentials , //if none we get them from env 
}
 
pub struct AppState {
    pub database: Arc<Mutex<Database>>,
 
    pub indexing_state: IndexingState ,
    
}
 
async fn collect_events( 
    mut app_state:AppState ,
    app_config: &AppConfig,
    chain_state: Arc<Mutex<ChainState>>
) -> AppState {

    
    let current_index_contract_name = &app_config.indexing_config.current_index_contract_name ;
    

    let Some(current_contract_config) =  app_config.contract_config_map.get(current_index_contract_name) else {
         warn!("contract config missing ");
         return app_state

    } ;


    let chain_id = current_contract_config.chain_id;


    let Some(rpc_uri) = app_config.rpc_uri_map.get( &chain_id ) else {
        warn!("no rpc uri");
        return app_state; 
    };

    let provider = Provider::<Http>::try_from(rpc_uri).unwrap( );
    
    
    let most_recent_block_number = match chain_state.lock().await.most_recent_block_number.clone(){
        
        Some(block_number) => block_number,
        None => {  
            
            //could not read recent block number so we cant continue 
            return app_state
        }
        
    };
    
    
    let chain_id = match chain_state.lock().await.chain_id.clone(){
        
        Some(chain_id) => chain_id,
        None => {  
            
            //could not read recent block number so we cant continue 
            return app_state
        }
        
    };
    
    
    
         
    let contract_address = Address::from_str(&current_contract_config.contract_address)
        .expect("Failed to parse contract address");
        
    let mut block_gap:u32 = match app_state.indexing_state.synced {
        true => {app_config.indexing_config.fine_block_gap }
        false => {app_config.indexing_config.course_block_gap }
    };
    
    
    
    
    if app_state.indexing_state.provider_failure_level > 0  {
        let block_gap_division_factor = std::cmp::min( 
            app_state.indexing_state.provider_failure_level , 8 );
        
        block_gap = std::cmp::min( 1 , block_gap / block_gap_division_factor );        
    }
    
    
    let Some(contract_abi )  = &app_config.contract_abi_map.get(current_index_contract_name) else {
        warn!("no abi ");
        return app_state ;
    }; 

  
    let start_block = app_state.indexing_state.current_indexing_block;
    
    
    
    //if we are synced up to 4 blocks from the chain head, skip collection. 
    if start_block > most_recent_block_number - 4 {
        info!( "Fully synced- skipping event collection {} {}" , start_block,most_recent_block_number );
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
        provider,
        chain_id.as_u64()
    ).await {
        Ok( evts ) => evts,
        Err(_e) => { 
                
           
            //we increase the failure level which will shrink the block gap to ease the load on the provider in case that was the issue
            app_state.indexing_state.provider_failure_level += 1 ; 
                            
              //max failure level is 8 
            app_state.indexing_state.provider_failure_level = std::cmp::min( 
                app_state.indexing_state.provider_failure_level , 8 );
              
            return app_state
        }       
    };
        
        
    //on success we reduce the failure level 
    if app_state.indexing_state.provider_failure_level >= 1 {
          app_state.indexing_state.provider_failure_level -= 1 ; 
    }
  
    

    
    for event_log in event_logs {
        
          info!("decoded event log {:?}", event_log);
          
         // let psql_db = &app_state.database;

          let mut psql_db = app_state.database.lock().await; // Lock database correctly
           
          let inserted = EventsModel::insert_one(&event_log, &mut *psql_db ).await;

          info!("inserted {:?}", inserted);

        //  if inserted.is_err_and( |e| e == PostgresModelError::Timeout ) {
         //   encountered_insertion_timeout = true ;
         // }

        
    }
    
   // if !encountered_insertion_timeout {
          //progress the current indexing block
          app_state.indexing_state.current_indexing_block = end_block + 1; 
  /*  }else {
        warn!("Encountered a timeout with the database- retrying");

        // Unlock database first before reconnecting
           // drop(psql_db);

            // Reconnect and update `app_state.database`
            let mut db_lock = app_state.database.lock().await;
            if let Err(e) = db_lock.reconnect(     ).await {
                eprintln!("Database reconnection failed: {:?}", e);
            } else {
                info!("Database reconnected successfully.");
            }
    }*/
    // there there was an error, we are going to cycle this period again .

  



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
    
    
    
    
    let contract_address = Address::from_str( &app_config.contract_config.contract_address ).unwrap();
    

    let configured_start_block:U64 = app_config.contract_config.start_block.clone().into(); 


     

     let most_recent_event_blocknumber = {
        let mut psql_db = app_state.database.lock().await;
        find_most_recent_event_blocknumber(contract_address, &mut psql_db).await
    }; // `psql_db` is dropped here to avoid deadlock later



    //our indexing start block is the configured block UNLESS there is a newer more recent event - then we skip ahead 
    let mut indexing_start_block = configured_start_block;

    if let Some(most_recent_bn) = most_recent_event_blocknumber {
        if most_recent_bn > indexing_start_block {
            indexing_start_block = most_recent_bn;
        }
    }
    
    
    app_state.indexing_state.current_indexing_block =  indexing_start_block; 
    

    info!("Initializing current indexing block {}" , app_state.indexing_state.current_indexing_block );

   
 
    let mut initialize_loop_interval = interval(Duration::from_secs(2));
    loop {
        
        let collect_most_recent_block =  collect_blockchain_data(
             &Arc::clone(&app_config_arc),
             
             ).await;
             
             info!("ask api for block state");
             
        if let Ok((block_number,chain_id)) = collect_most_recent_block {
            
            chain_state.lock().await.most_recent_block_number = Some( block_number );
            chain_state.lock().await.chain_id = Some(chain_id);
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
    
    
    
    let mut collect_events_interval = interval(Duration::from_millis( 
         app_config.indexing_config.index_rate ));
    
    let mut collect_blockchain_data_interval = interval( Duration::from_millis(
         app_config.indexing_config.update_block_number_rate ) );
  
     loop {
        select! {
            _ = collect_events_interval.tick() => {
                app_state = collect_events(
                    app_state,
                     &Arc::clone(&app_config_arc), 
                     Arc::clone(&chain_state)).await;
            }
            _ = collect_blockchain_data_interval.tick() => {
                if let Ok((block_number, chain_id)) = collect_blockchain_data(
                     &Arc::clone(&app_config_arc), 
                     ).await{
                           chain_state.lock().await.most_recent_block_number = Some( block_number );
                           chain_state.lock().await.chain_id = Some( chain_id );
                     }
            }
        }
    }
    

}




async fn collect_blockchain_data(  
     app_config: &AppConfig, 
   //  chain_state: Arc<Mutex<ChainState>>
      ) -> Result< (U64, U256), ProviderError> {
    
     let rpc_uri = &app_config.indexing_config.rpc_uri;

     let provider = Provider::<Http>::try_from(rpc_uri).unwrap( );
    
     let block_number = provider.get_block_number().await?;
     info!("Current block number: {}", block_number);
        
     let chain_id = provider.get_chainid().await?;
        
   
    
     Ok((block_number, chain_id.into()))
}



