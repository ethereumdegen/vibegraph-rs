

 









use degen_sql::db::postgres::postgres_db::DatabaseCredentials;
use vibegraph::{IndexingConfig, ContractConfig, AppConfig, Vibegraph};


 
use dotenvy::dotenv;


use serde_json;

 



   
  

/*

cargo run config/payspec_config.json

*/

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().expect(".env file not found");

    let rpc_uri =  std::env::var("RPC_URL")
        .expect("RPC_URL must be set");

    
    let indexing_config = IndexingConfig {
         rpc_uri,
         index_rate: 4_000, //ms
         update_block_number_rate: 40_000,  //ms
         course_block_gap: 2000,
         fine_block_gap: 100,
         safe_event_count: 400,  //not used for now 

    };
    
    let path = std::env::args().nth(1).unwrap_or("config/payspec_config.json".into());
    let config_content = std::fs::read_to_string(path).expect("Could not read config");
    
    let contract_config: ContractConfig = serde_json::from_str(&config_content).expect("Could not parse config");
    
    //let db_conn_url = DatabaseCredentials::from_env()  ;

    let db_conn_url = std::env::var("DB_CONN_URL")
        .expect("RPC_URL must be set");


    
     let  app_config = AppConfig {
        
        indexing_config,
        contract_config,
        db_conn_url ,
       
    };
    
    
    
    Vibegraph::init( &app_config ).await;
    
    
}

 