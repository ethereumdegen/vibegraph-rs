 
 
 ##  Vibegraph-rs  
   
   An implementation of vibegraph for rust-lang
   Vibegraph reads ethereum events from a lightweight RPC and caches them to a database. 
   
  
 
  ### TODO 
  
  1. Add an interval which fetches the current eth blocknumber so we know if we are synced up 
  
  2. Store logs to sql database 
  
  3. add the 'expansion factor' to account for provider failures (give provider less load) 
  
  
  