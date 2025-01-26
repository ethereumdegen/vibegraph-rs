use degen_sql::db::postgres::postgres_db::{Database, DatabaseCredentials};
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
      dotenv().ok();
      let credentials = DatabaseCredentials::from_env();


      let conn_url = credentials.build_connection_url();
    
      println!("migrating db: {}", conn_url);



    let mut database = Database::connect(credentials, None).await?;

    let _migration = database.migrate().await?;

    Ok(())
}