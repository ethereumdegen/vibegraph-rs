
use tokio_postgres::{Error as PostgresError};

 
use super::{super::postgres_db::Database};






 
 
 
 
use ethers::types::Address;
use ethers::utils::to_checksum;
 
pub struct UsersModel {
    
    
}
 
 impl UsersModel {
     
     
      pub async fn insert_new_user ( 
         public_address: Address, 
         psql_db: &Database
     ) -> Result<  i32   , PostgresError> {
         
         let formatted_address = to_checksum(&public_address,None).to_string();
        
         
           let insert_result = psql_db.query_one("
            INSERT INTO users 
            ( public_address  ) 
            VALUES ( $1  )
             ON CONFLICT(public_address) DO NOTHING
            RETURNING id;
            ", 
            &[&formatted_address.clone() ]).await ;
            
          
            match insert_result {
                    Ok(row) => Ok(row.get(0)),  // Successfully inserted new product and got its ID.
                    Err(_) => {
                        // Conflict occurred and no new product was inserted. Fetch the existing product's ID.
                        let existing_row = psql_db.query_one(
                         "SELECT id users products WHERE public_address = $1", 
                         &[& formatted_address.clone()]).await?;
                         
                        Ok(existing_row.get(0))
                    }
                } 
                
       //  Ok(  formatted_address   )
     }
     
     
 }
 
 
  