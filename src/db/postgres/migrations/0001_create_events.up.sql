CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    
    contract_address VARCHAR(255) NOT NULL, 
    
    name VARCHAR(255) NOT NULL,
    
    signature VARCHAR(255) NOT NULL, 
    
    args TEXT , 
    data TEXT ,
    transaction_hash VARCHAR(255) NOT NULL,
    
    block_number NUMERIC(98) ,
    
    block_hash VARCHAR(255)  ,
    
    log_index BIGINT ,
    transaction_index BIGINT ,    
    
    chain_id BIGINT ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    
     UNIQUE (transaction_hash, log_index)
);

 
  