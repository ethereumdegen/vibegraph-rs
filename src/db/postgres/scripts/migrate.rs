use degen_sql::db::postgres::postgres_db::{Database, DatabaseCredentials};
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
      dotenv().ok();
     

      //  let credentials = DatabaseCredentials::from_env();

      // let conn_url = credentials.build_connection_url();

           let db_conn_url =  std::env::var(  "DB_CONN_URL"  )
        .expect(" DB_CONN_URL must be set in env ");

      
    
      println!("migrating db: {}", db_conn_url);



    let mut database = Database::new(db_conn_url, 2,  None)?;

    let _migration = database.migrate().await?;

    Ok(())
}