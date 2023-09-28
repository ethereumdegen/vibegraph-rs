
/*
    postgresql
  

    use diesel ?   for the migrations and structuring ? 
    


    You may want to explore different patterns for sharing resources, 
    such as using an Arc<Mutex<Database>> to safely share the
     connection between multiple tasks.
*/

// const databaseURL = useTestDb ?  `postgres://postgres:postgres@localhost` : `postgres://${DATABASE_USERNAME}:${DATABASE_PASSWORD}@${DATABASE_HOST}/${DATABASE_NAME}`

use tokio;
use tokio_postgres::{NoTls, Error as PostgresError};

 
use tokio_postgres_migration::Migration;
use std::error::Error;


use dotenvy::dotenv;
use std::env;

use std::str;


use include_dir::{include_dir, Dir};

const MIGRATIONS: Dir = include_dir!("./src/db/postgres/migrations");



//use no_comment::{IntoWithoutComments as _, languages};
 



struct Migrations{
    up: Vec<(&'static str, &'static str)>,
    down: Vec<(&'static str, &'static str)>
}

 
pub struct Database {
    pub client: tokio_postgres::Client,
}


enum PostgresInputType {
    Query,
    QueryOne,
    Execute
}

struct PostgresInput<'a> {
    input_type: PostgresInputType,
    query: String,
    params: &'a [&'a(dyn tokio_postgres::types::ToSql + Sync)]

}

impl Database {

    pub fn get_connection_url() -> String {
      
        dotenv().expect(".env file not found");

        let db_name = env::var("DB_NAME").unwrap();
        let db_host = env::var("DB_HOST").unwrap();
        let db_user = env::var("DB_USER").unwrap();
        let db_password = env::var("DB_PASSWORD").unwrap();


        return Database::build_connection_url ( db_user, db_password, db_host, db_name);
    }

    pub fn get_local_connection_url() -> String {
      
        dotenv().expect(".env file not found");

        let db_name = "postgres".to_string();
        let db_host = "localhost".to_string();
        let db_user = "postgres".to_string();
        let db_password = "postgres".to_string();


        return Database::build_connection_url ( db_user, db_password, db_host, db_name);
    }

    pub fn build_connection_url(db_user:String,db_password:String,db_host:String,db_name:String) -> String {
        return format!("postgres://{}:{}@{}/{}",  db_user, db_password, db_host, db_name);
    }

   

    pub async fn connect( ) -> Result<Database, PostgresError> {
        
      
          // Define the connection URL.
        let conn_url = Database::get_connection_url();

        println!("conn 1");

        println!("{}",conn_url);
        
        let (client, connection) =
        tokio_postgres::connect(&conn_url, NoTls).await?;


        /*let (client, connection) = tokio_postgres::connect(
            &format!("host={} user={} password={} dbname={}", db_host, db_user, db_password, db_name), NoTls).await?;
*/


        println!("conn 2");


        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            println!("starting connection to postgres db");

            //this is a blocking call i think !!!
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            } 

           
        });
 


        Ok(Database { client })
    }


    fn read_migration_files() -> Migrations {

        let mut migrations = Migrations{
            up: Vec::new(), 
            down: Vec::new()
            };
     

        for file in MIGRATIONS.files() {
            let path = file.path();
            let filename = path.file_stem().unwrap().to_str().unwrap();

            let filename_without_extension: &str = filename.split('.').next().unwrap();

            let contents = str::from_utf8(file.contents()).unwrap();
    
 
     

            println!("File name: {}", filename);
           

            if filename.contains(".down")  {
                println!("File contents: {}", contents);
                migrations.down.push((filename_without_extension, contents));
            }
 
            if filename.contains(".up")  {
                println!("File contents: {}", contents);
                migrations.up.push((filename_without_extension, contents));
            }
 
           
        }

        return migrations;
    }

    pub async fn migrate(&mut self) -> Result<(), Box<dyn Error>> {
      

        


        let client = &mut self.client; 
        
        let mut migrations:Migrations = Database::read_migration_files();  
         
        for up_migration in migrations.up.drain(..) {
                 println!("migrating {} {} ", up_migration.0, up_migration.1);
                let migration = Migration::new("migrations".to_string());
             
                // execute non existing migrations
                migration.up( client, &[up_migration] ).await?;
            
        }

       
        
        // ...
        Ok(())
    }


    //basically need to do the DOWN migrations and also delete some records from the migrations table  
    pub async fn rollback(&mut self) -> Result<(), Box<dyn Error>> {

        Ok(())
    }

    pub async fn rollback_full(&mut self) -> Result<(), Box<dyn Error>> {

        let mut migrations:Migrations = Database::read_migration_files();
         

        let client = &mut self.client; 
        
         for down_migration in migrations.down  .drain(..) .rev() {
               println!("migrating {}", down_migration.0);
                let migration = Migration::new("migrations".to_string());
                // execute non existing migrations
                migration.down( client, &[down_migration]).await?;
         }

        Ok(())
    }

/*
    pub async fn query(&self, query: &str) -> Result<Vec<tokio_postgres::Row>, PostgresError> {
        let rows = self.client.query(query, &[]).await?;
        Ok(rows)
    }*/
    pub async fn query(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<tokio_postgres::Row>, PostgresError> {
        let rows = self.client.query(query, params).await?;
        Ok(rows)
    }

    pub async fn query_one(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<tokio_postgres::Row, PostgresError> {
        let rows = self.client.query_one(query, params).await?;
        Ok(rows)
    }

    //use for insert, update, etc 
    pub async fn execute(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64, PostgresError> {
        let rows = self.client.execute(query, params).await?;
        Ok(rows)
    }





    
    async fn atomic_transaction(&mut self, steps: Vec<PostgresInput<'_>>) -> Result<(), PostgresError> {

        // Start a transaction
        let transaction = self.client.transaction().await?;
     

        //for each step in steps 
        for step in steps {
            //execute the step
            let result = transaction.execute(&step.query, step.params).await;
            //check if the result is ok
            if result.is_err() {
                //if not rollback
                transaction.rollback().await?;
                //return error
                return Err(PostgresError::from(result.err().unwrap()));
            }
        }

        //if all steps are ok commit
        transaction.commit().await?;
        //return ok
        Ok(())
  
    }
}



pub fn try_get_option<'a, T: tokio_postgres::types::FromSql<'a>>(row: &'a tokio_postgres::Row, column: &str) -> Option<T> {
    match row.try_get::<&str, T>(column) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}