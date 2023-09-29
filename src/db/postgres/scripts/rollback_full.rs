
use inquire::Confirm;

use vibegraph::db::postgres::postgres_db::Database;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

 

    let ans = Confirm::new("Are you sure you want to roll back?")
    .with_default(false) 
    .prompt();

    match ans {
        Ok(true) =>  {
            let mut database = Database::connect().await?;

            let _migration = database.rollback_full().await?;

            println!("Rollback complete");

        },
        Ok(false) =>  {
            println!("Rollback operation cancelled");

        },
        Err(_) =>  {
            println!("Rollback operation cancelled");


        },
    }
 
    Ok(())
}