CREATE TABLE network_data (
    id SERIAL PRIMARY KEY,

    
    chain_id BIGINT ,

    latest_block_number BIGINT ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW() ,



     UNIQUE (chain_id)
     
);

 
  