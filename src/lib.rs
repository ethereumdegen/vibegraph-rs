


 
use crate::db::postgres::models::event_indexer_model::EventIndexerModel;
use db::postgres::models::event_indexer_model::EventIndexer;
use db::postgres::models::network_data_model::NetworkData;
use thiserror::Error;

use std::collections::HashMap;
use degen_sql::db::postgres::models::model::PostgresModelError;
use ethers::providers::{ProviderError};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use ethers::prelude::{
     Provider, Middleware};
use ethers::types::{Address, U64, U256};
use event::{read_contract_events,  };

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

pub mod rpc_network;

 


pub struct Vibegraph {
  
}



impl Vibegraph {
    
    
    
 ///Used to externally start vibegraph 
pub async fn init (
   
    app_config: &AppConfig  
    
){
    
        let indexing_state = Arc::new(Mutex::new( IndexingState::default() ));

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
         //   indexing_state,
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
        
        //let chain_state = Arc::new(Mutex::new(ChainState::default()));
        
        
        app_state = initialize(app_state, &app_config, /*&Arc::clone(&chain_state)*/).await;
    
        start(app_state, &Arc::clone(&indexing_state), &app_config,).await;
        
        
        
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

   pub current_indexer_id: Option<i32>,
    
    pub provider_failure_level: u32,

    pub current_network_index: usize, 

}


impl Default for IndexingState {

    fn default() -> Self {
        Self {
          //  current_indexing_block: 0.into(),
          //  synced: false,

            current_indexer_id : None, 

            provider_failure_level: 0,
            current_network_index:0 
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

    pub network_chain_ids: Vec<u64>,
    
   // pub updating_network_chain_id: u32, //start at 1 ? 

    
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
 
     
}
 
async fn collect_events( 
    mut app_state:AppState ,
    app_config: &AppConfig,

    indexing_state: &Arc<Mutex<IndexingState>>,

    event_indexer_id: &i32,
    event_indexer: &EventIndexer , 
  //   chain_state: Arc<Mutex<ChainState>>
) -> AppState {

    info!( "collecting events ... " );
   
    
    let current_index_contract_name = &event_indexer.contract_name ;
    

  /*  let Some(current_contract_config) =  app_config.contract_config_map.get(current_index_contract_name) else {
         warn!("contract config missing ");
         return app_state

    } ;*/


    let chain_id =  &event_indexer.chain_id; 
    let contract_address = &event_indexer.contract_address;

    let   current_indexing_block =  event_indexer.current_indexing_block .clone();
    let   start_block =  event_indexer.start_block .clone();
  
    let Some(rpc_uri) = app_config.rpc_uri_map.get( &chain_id ) else {
        warn!("no rpc uri");
        return app_state; 
    };

    let provider = Provider::<Http>::try_from(rpc_uri).unwrap( );
    

     let mut psql_db = app_state.database.lock().await; // Lock database correctly
       

    let most_recent_block_number_result =   NetworkData::find_one_by_chain_id(
            chain_id.clone() as i64,
            &mut psql_db
        ).await .ok() .clone() ; 

    drop(psql_db);


    // need to read this from db !!     
    let most_recent_block_number = match  most_recent_block_number_result{
        
        Some(network_data) => network_data.latest_block_number.clone() as u64,
        None => {  
            
            //could not read recent block number so we cant continue 
            return app_state
        }
        
    };
    
    
 
    
   let is_synced = event_indexer.synced.clone();  
   
        
    let mut block_gap:u32 = match  is_synced {
        true => {app_config.indexing_config.fine_block_gap }
        false => {app_config.indexing_config.course_block_gap }
    };
    
    
    
    let mut provider_failure_level =  indexing_state.lock().await .provider_failure_level.clone() ; 
    if   provider_failure_level  > 0  {
        let block_gap_division_factor = std::cmp::min( 
             provider_failure_level  , 8 );
        
        block_gap = std::cmp::min( 1 , block_gap / block_gap_division_factor );        
    }
    
    
    let Some(contract_abi )  = &app_config.contract_abi_map.get(current_index_contract_name) else {
        warn!("no abi:  {}", current_index_contract_name);
        return app_state ;
    }; 

  
    let start_block =  current_indexing_block.unwrap_or( start_block );



    let mut new_is_synced = false;  
    

    // ---------------------  
   
    
    //if we are synced up to 4 blocks from the chain head, skip collection. 
    if start_block > most_recent_block_number - 4 {
        info!( "Fully synced- skipping event collection {} {}" , start_block,most_recent_block_number );
        return app_state
    }
 
     info!("index starting at {}", start_block);
    
    
    let mut end_block = start_block + std::cmp::max(block_gap as u64 - 1, 1);
    
    if end_block >= most_recent_block_number {
        end_block = most_recent_block_number;
        new_is_synced = true;
    }

    let event_logs = match read_contract_events(
        *contract_address,
        contract_abi,
        start_block.into(),
        end_block.into(),
        provider,
        *chain_id 
    ).await {
        Ok( evts ) => evts,
        Err(_e) => { 
                
           
            //we increase the failure level which will shrink the block gap to ease the load on the provider in case that was the issue
          //  app_state.indexing_state.provider_failure_level += 1 ; 

            provider_failure_level += 1; 

             indexing_state.lock().await.provider_failure_level = std::cmp::min( 
                provider_failure_level , 8 );
                            
              //max failure level is 8 
          /*  app_state.indexing_state.provider_failure_level = std::cmp::min( 
                app_state.indexing_state.provider_failure_level , 8 );*/
              
            return app_state
        }       
    };
        
        
    //on success we reduce the failure level 
    if provider_failure_level >= 1 {
          provider_failure_level -= 1 ; 
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




            // TODO :NEED TO UPDATE SQL RECORD WITH THIS !! 
          let new_current_indexing_block =  end_block + 1; 
          info!("new_current_indexing_block {}", new_current_indexing_block);

          let mut psql_db = app_state.database.lock().await; // Lock database correctly
          let _ = EventIndexerModel::update_current_indexing_block( *event_indexer_id , new_current_indexing_block, &mut psql_db).await;
          let _ = EventIndexerModel::update_is_synced( *event_indexer_id , new_is_synced, &mut psql_db).await;
          drop (psql_db);

          //update the provider failure level in state 
          indexing_state.lock().await.provider_failure_level = std::cmp::min( 
                provider_failure_level , 8 );


          //indexing_state.lock().await.current_indexing_block = end_block + 1; 
 
    // there there was an error, we are going to cycle this period again .

  
 



    app_state
} 



 
 

async fn initialize(
    mut app_state: AppState, 
    app_config: &AppConfig ,
   // chain_state: &Arc<Mutex<ChainState>>
    
    ) -> AppState {
 
    
     
    //initialize state 

      
    let app_config_arc = Arc::new( &app_config );
    
    
    info!("Initializing state");
    
    
    /*
    
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
*/
   
   /*
 
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
    */


    info!("Initialization complete");
    

   
    app_state 

}

 
   

async fn start(
    mut app_state: AppState, 
    indexing_state: &Arc<Mutex<IndexingState>>,

    app_config: &AppConfig ,
    //chain_state: &Arc<Mutex<ChainState>>
    
    ){
 
    
     
    let app_config_arc = Arc::new( &app_config );
    
    
    
    let mut collect_events_interval = interval(Duration::from_millis( 
         app_config.indexing_config.index_rate ));
    
    let mut collect_blockchain_data_interval = interval( Duration::from_millis(
         app_config.indexing_config.update_block_number_rate ) );
  
     loop {
        select! {
            _ = collect_events_interval.tick() => {




                let mut current_indexer_id = indexing_state.lock().await.current_indexer_id .clone() ;

                  let mut psql_db = app_state.database.lock().await;

                let next_indexer_result  = EventIndexerModel::find_next_event_indexer(
                     current_indexer_id ,
                     &mut psql_db 
                 ).await ;


             
               let (next_indexer_id,  next_indexer_data ) =  if let Ok( ( id,   ref indexer_data))   = next_indexer_result {
 
                  //  current_indexer_id = Some(id) ;

                     ( Some( id.clone() ) , Some( indexer_data.clone() )  )

                }else { 

                     //  current_indexer_id = None ;  

                       ( None, None )
                };

                drop(psql_db);
                drop(next_indexer_result);

                if let Some( next_indexer_id ) = next_indexer_id {

                    if let Some(next_indexer_data) = next_indexer_data {
                       app_state = collect_events(
                        app_state,
                         &Arc::clone(&app_config_arc), 
                         &Arc::clone( & indexing_state ),

                         &next_indexer_id, 
                         &next_indexer_data ,
                        // Arc::clone(&chain_state)
                        ).await;
                     }


                indexing_state.lock().await.current_indexer_id  = Some( next_indexer_id ) ;
              }else {
                indexing_state.lock().await.current_indexer_id  =  None  ; //reset 

              }

                

            }
            _ = collect_blockchain_data_interval.tick() => {


                let mut current_network_index = indexing_state.lock().await.current_network_index .clone() ;

                info!("current_network_index {}", current_network_index );


                  if let Ok((block_number, chain_id)) = collect_blockchain_data(
                     &Arc::clone(&app_config_arc), 
                      & current_network_index , 

                     ).await{

                      
                         let mut psql_db = app_state.database.lock().await;

                         let latest_block_number =  block_number.as_u64()  as i64 ;
                         let chain_id = chain_id.as_u64()  as i64; 
                         let _ = NetworkData::insert( chain_id, latest_block_number,  &mut psql_db ).await;
                         //store in database 

                        
                     } //end if 


                    current_network_index += 1 ;

                    let max_network_index = app_config.indexing_config.network_chain_ids.len();
                    if current_network_index >= max_network_index {
                        current_network_index = 0;
                    }

                    indexing_state.lock().await.current_network_index  = current_network_index;

            }
        }
    }
    

}



 
#[derive(Error, Debug)]
pub enum VibegraphError {
  #[error("Error with the data provider: {0}")]
    ProviderError(#[from] ProviderError),

    #[error("Error with parse: {0}")]
    ParseError(#[from] url::ParseError),


    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("RPC URL not found for the chain ID")]
    RpcUrlNotFoundError,

    #[error("Unknown error occurred")]
    UnknownError,
} 





async fn collect_blockchain_data(  
     app_config: &AppConfig, 
     network_index: & usize , 
   //  chain_state: Arc<Mutex<ChainState>>
      ) -> Result< (U64, U256), VibegraphError> {

     let current_chain_id = app_config.indexing_config.network_chain_ids.get(*network_index).unwrap_or(&1);

    let rpc_uri = app_config.rpc_uri_map.get(current_chain_id)
        .ok_or(VibegraphError::RpcUrlNotFoundError)?;

    let provider = Provider::<Http>::try_from(rpc_uri.as_str())
        .map_err(|e| VibegraphError::ParseError(e) )?;

    let block_number = provider.get_block_number().await
        .map_err(VibegraphError::ProviderError)?;
    info!("Current block number: {}", block_number);

    let chain_id = provider.get_chainid().await
        .map_err(VibegraphError::ProviderError)?;

    Ok((block_number, chain_id.into()))

    /*
     let current_chain_id = app_config.indexing_config.network_chain_ids.get( *network_index ) .unwrap_or( &1 );
    
     let rpc_uri =  app_config.rpc_uri_map.get(  current_chain_id  ) .ok_or( Err( VibegraphError::ConfigError( "Rpc Map Error".to_string()) )  )?;

     let provider = Provider::<Http>::try_from(rpc_uri) ?;
    
     let block_number = provider.get_block_number().await?;
     info!("Current block number: {}", block_number);
        
     let chain_id = provider.get_chainid().await?;
        
   
    
     Ok((block_number, chain_id.into()))*/
}



