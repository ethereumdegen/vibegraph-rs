CREATE TABLE event_indexers (
    id SERIAL PRIMARY KEY,
    
    
    name VARCHAR(255) NOT NULL,
    contract_name VARCHAR(255) NOT NULL,
    contract_address VARCHAR(255) NOT NULL,     
    
    chain_id BIGINT  NOT NULL ,

    start_block BIGINT NOT NULL , 


    current_indexing_block BIGINT, 
    synced BOOL NOT NULL DEFAULT false,




    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW() ,
    
    UNIQUE (name)
);

 
  