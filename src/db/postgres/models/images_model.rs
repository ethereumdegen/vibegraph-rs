

use ethers::{types::Address, utils::to_checksum};

use crate::db::postgres::postgres_db::Database;

use super::model::PostgresModelError;
 
   

pub struct ImagesModel {
    
    
}


impl ImagesModel { 
     
     
     
     pub async fn find_or_create ( 
         hash: &String, 
         parent_public_address: &Address, 
         psql_db: &Database
     ) -> Result<  i32   , PostgresModelError> {
          
           let formatted_address = to_checksum(&parent_public_address,None).to_string();
        
           
           let insert_result = psql_db.query_one("
            INSERT INTO images 
            ( hash, parent_public_address  ) 
             VALUES ( $1 , $2  ) 
              
             RETURNING id;
            ", 
            &[&hash , &formatted_address ]).await ;
            
          
            match insert_result {
                    Ok(row) => Ok(row.get(0)),  // Successfully inserted new product and got its ID.
                    Err(e) => {
                       eprintln!("Database error: {:?}", e);
                        
                        // Conflict occurred and no new product was inserted. Fetch the existing product's ID.
                        let existing_row = psql_db.query_one(
                         "SELECT id FROM images WHERE hash = $1", &[&hash]).await?;
                        Ok(existing_row.get(0))
                    }
                } 
     }
     
    
}