 
 
 ##  Vibegraph-rs  
   
A microservice application that reads ethereum contract events from a lightweight RPC and caches them to a database (Postgres). 

```
cargo add vibegraph
```   
___
   
   
 ###  Quickstart
   ```
   cargo run config/payspec_config.json   
   ```
   ___
   
   Example config: 
   ``` 
    
     let  app_config = AppConfig {
        
        indexing_config,
        contract_config 
       
    }; 
    
    Vibegraph::init( &app_config ).await;
    
   ```
   
   
   Example output: 
   ``` 
   
[2023-09-28T19:42:47Z INFO  vibegraph] index starting at 4286418
[2023-09-28T19:42:52Z INFO  vibegraph] index starting at 4288418
[2023-09-28T19:42:57Z INFO  vibegraph] index starting at 4290418
[2023-09-28T19:43:02Z INFO  vibegraph] index starting at 4292418
[2023-09-28T19:43:02Z INFO  vibegraph] decoded event log ContractEvent { name: "Transfer", signature: 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, args: [LogParam { name: "from", value: Address(0xf637ce9928a9e6632920338b98f2543ea0b5526f) }, LogParam { name: "to", value: Address(0x5a5b978142c8f08dd013901b50892bac49f3b700) }, LogParam { name: "tokenId", value: Uint(0) }], address: 0x4590383ae832ebdfb262d750ee81361e690cfc9c, data: [], transaction_hash: Some(0x5c9e7eb1aa7fe6564eeeaef6e251c44c80bd4c0e059818f548edc744f9bae999), block_number: Some(4294096), block_hash: Some(0x384e1d61b85d7e5a41a8a2e490b7e019a707301809f18508a004768ced29b94b), log_index: Some(57), transaction_index: Some(39) }
  ```

  ___
  
  ![image](https://github.com/ethereumdegen/vibegraph-rs/assets/6249263/85bbdf4b-fcab-49e4-884b-65f038100381)
  _Event logs stored to Supabase (PSQL)_

 ___
