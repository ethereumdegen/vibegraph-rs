use ethers::abi::Address;
use vibegraph::db::postgres::models::event_indexer_model::EventIndexer;
use vibegraph::db::postgres::models::event_indexer_model::EventIndexerModel;
use degen_sql::db::postgres::postgres_db::{Database, DatabaseCredentials};
use dotenvy::dotenv;
use vibegraph::rpc_network::RpcNetwork;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
      dotenv().ok();
     

      //  let credentials = DatabaseCredentials::from_env();

      // let conn_url = credentials.build_connection_url();

           let db_conn_url =  std::env::var(  "DB_CONN_URL"  )
        .expect(" DB_CONN_URL must be set in env ");

      
    
      println!("add event indexers: {}", db_conn_url);



    let mut database = Database::new(db_conn_url, None) ?;

    let lender_pools_factory_arbitrum = EventIndexer::new(
      "lender_pools_factory_arbitrum".to_string(),
       "lender_pools_factory".to_string(),
      "0xC2a093B641496Ac8AA9d6a17f216ADF4a42FC9B6".parse::<Address>().unwrap(),

      RpcNetwork::Arbitrum.get_chain_id(),  
      292978933 
     );

    let table_name = "event_indexers"; 

    let _ = EventIndexerModel::insert_one( table_name.to_string() , &lender_pools_factory_arbitrum , &  database ).await; 
 

     // EventIndexer creation and insertion for Tellerv2 on Arbitrum
    let tellerv2_arbitrum = EventIndexer::new(
        "tellerv2_arbitrum".to_string(),
         "tellerv2".to_string(),
        "0x5cfD3aeD08a444Be32839bD911Ebecd688861164".parse::<Address>().unwrap(),
        RpcNetwork::Arbitrum.get_chain_id(),
        108629279,
    );
    let _ = EventIndexerModel::insert_one(table_name.to_string() , &tellerv2_arbitrum, &  database).await;

    // EventIndexer creation and insertion for Lender Pools Factory on Base
    let lender_pools_factory_base = EventIndexer::new(
        "lender_pools_factory_base".to_string(),
         "lender_pools_factory".to_string(),
        "0x7FBCefE4aE4c0C9E70427D0B9F1504Ed39d141BC".parse::<Address>().unwrap(),
        RpcNetwork::Base.get_chain_id(),
        24824438,
    );
    let _ = EventIndexerModel::insert_one(table_name.to_string() , &lender_pools_factory_base, &  database).await;

    // EventIndexer creation and insertion for Tellerv2 on Base
    let tellerv2_base = EventIndexer::new(
        "tellerv2_base".to_string(),
           "tellerv2".to_string(),
        "0x5cfD3aeD08a444Be32839bD911Ebecd688861164".parse::<Address>().unwrap(),
        RpcNetwork::Base.get_chain_id(),
        2935370,
    );
    let _ = EventIndexerModel::insert_one(table_name.to_string() , &tellerv2_base, &  database).await;

    // EventIndexer creation and insertion for Lender Pools Factory on Mainnet
    let lender_pools_factory_mainnet = EventIndexer::new(
        "lender_pools_factory_mainnet".to_string(),
         "lender_pools_factory".to_string(),
        "0x0848E884b2DBb63727aa3216b921C279f6DC9a91".parse::<Address>().unwrap(),
        RpcNetwork::Mainnet.get_chain_id(),
        21616926,
    );
    let _ = EventIndexerModel::insert_one(table_name.to_string() , &lender_pools_factory_mainnet, &  database).await;

    // EventIndexer creation and insertion for Tellerv2 on Mainnet
    let tellerv2_mainnet = EventIndexer::new(
        "tellerv2_mainnet".to_string(),
           "tellerv2".to_string(),
        "0x00182FdB0B880eE24D428e3Cc39383717677C37e".parse::<Address>().unwrap(),
        RpcNetwork::Mainnet.get_chain_id(),
        15094701,
    );
    let _ = EventIndexerModel::insert_one(table_name.to_string() , &tellerv2_mainnet, &  database).await;

    // EventIndexer creation and insertion for Lender Pools Factory on Polygon
    let lender_pools_factory_polygon = EventIndexer::new(
        "lender_pools_factory_polygon".to_string(),
         "lender_pools_factory".to_string(),
        "0x2fF5ea5CF5061EB0fcfB7A2AafB8CCC79f3F73ea".parse::<Address>().unwrap(),
        RpcNetwork::Polygon.get_chain_id(),
        66265322,
    );
    let _ = EventIndexerModel::insert_one(table_name.to_string() , &lender_pools_factory_polygon, &  database).await;

    // EventIndexer creation and insertion for Tellerv2 on Polygon
    let tellerv2_polygon = EventIndexer::new(
        "tellerv2_polygon".to_string(),
           "tellerv2".to_string(),
        "0xD3D79A066F2cD471841C047D372F218252Dbf8Ed".parse::<Address>().unwrap(),
        RpcNetwork::Polygon.get_chain_id(),
        26017630,
    );
    let _ = EventIndexerModel::insert_one(table_name.to_string() , &tellerv2_polygon, &  database).await;





    Ok(())
}