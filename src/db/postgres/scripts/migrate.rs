 
 
use vibegraph::db::postgres::postgres_db::Database;
 

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    let mut database = Database::connect().await?;

    let _migration = database.migrate().await?;

    println!("Migration complete");

   /* let config = Config {
        database_url:  Database::get_connection_url(),
        migrations_directory: "../migrations",
        ..Default::default()
    };

    let mut migrator = Migrator::new(NoTls, config).await?;

    // Run all migrations
    migrator.run().await?;

    // List all migrations
    let migrations: Vec<Migration> = migrator.list().await?;
    for migration in migrations {
        println!("Migration: {}", migration);
    }*/

    Ok(())
}