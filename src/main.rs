

 









use std::env;
use ethers::abi::Abi;
use std::fs;
use serde_json::from_str;
use std::collections::HashMap;
use degen_sql::db::postgres::postgres_db::DatabaseCredentials;
use ethers::core::k256::pkcs8::der;
use vibegraph::{rpc_network::RpcNetwork, AppConfig, ContractConfig, IndexingConfig, Vibegraph};


 
use dotenvy::dotenv;


use serde_json;

 



   
  

/*

cargo run config/payspec_config.json

*/

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().expect(".env file not found");

    //let rpc_uri =  std::env::var("RPC_URL")
    //    .expect("RPC_URL must be set");

    

    let networks = vec![
     RpcNetwork::Mainnet,
     RpcNetwork::Base,
     RpcNetwork::Arbitrum,
     RpcNetwork::Polygon, 
    ];

     let chain_ids: Vec<u64> = networks.iter().map(|n| n.get_chain_id()).collect();

 


    let indexing_config = IndexingConfig {
        // rpc_uri,
         index_rate: 4_000, //ms
         update_block_number_rate: 40_000,  //ms
         course_block_gap: 2000,
         fine_block_gap: 100,
         safe_event_count: 400,  //not used for now 

         network_chain_ids: chain_ids.clone(),


    };
    
  //  let config_folder_path =  "config/".to_string() ;
    let abi_folder_path =  "abi/".to_string() ;


 

    let db_conn_url = std::env::var("DB_CONN_URL")
        .expect("RPC_URL must be set");


   
  //  let (contract_config_map, chain_ids) = load_contract_configs(&config_folder_path);
    let contract_abi_map = load_contract_abis(&abi_folder_path);
    let rpc_uri_map = load_rpc_uris(&chain_ids);


 

    
     let  app_config = AppConfig {
        
        indexing_config,
        contract_abi_map,
        rpc_uri_map,
    //    contract_config_map,
        db_conn_url ,
        event_indexer_table_name: None ,
    };
    
    
    
    Vibegraph::init( &app_config ).await;
    
    
}

 



pub fn load_contract_configs(config_folder_path: &str) -> (HashMap<String, ContractConfig>, Vec<u64>) {
    let mut contract_config_map = HashMap::new();
    let mut chain_ids = Vec::new();

    let paths = fs::read_dir(config_folder_path).expect("Failed to read config directory");
    for path in paths {
        let path = path.expect("Failed to read path in directory").path();
        if path.is_file() {
            let config_content = fs::read_to_string(&path).expect("Could not read config file");
            let contract_config: ContractConfig = from_str(&config_content).expect("Could not parse config");
            chain_ids.push(contract_config.chain_id);
            contract_config_map.insert(path.to_str().unwrap().to_string(), contract_config);
        }
    }

    (contract_config_map, chain_ids)
}

pub fn load_contract_abis(config_folder_path: &str) -> HashMap<String, Abi> {
    let mut contract_abi_map = HashMap::new();

    let paths = fs::read_dir(config_folder_path).expect("Failed to read ABI directory");
    for path in paths {
        let path = path.expect("Failed to read path in directory").path();
        if path.is_file() {
            let abi_content = fs::read_to_string(&path).expect("Could not read ABI file");
            let contract_abi: Abi = from_str(&abi_content).expect("Could not parse ABI");
            contract_abi_map.insert(path.file_stem().unwrap().to_str().unwrap().to_string(), contract_abi);
        }
    }

    contract_abi_map
}

pub fn load_rpc_uris(chain_ids: &[u64]) -> HashMap<u64, String> {
    let mut rpc_uri_map = HashMap::new();

    for &chain_id in chain_ids {
        let rpc_network = RpcNetwork::from_chain_id(chain_id);
        if let Some(network) = rpc_network {
            let rpc_url_env_var = network.get_rpc_url_env_var();
            if let Ok(rpc_url) = env::var(rpc_url_env_var) {
                rpc_uri_map.insert(chain_id, rpc_url);
            }
        }
    }

    rpc_uri_map
}
