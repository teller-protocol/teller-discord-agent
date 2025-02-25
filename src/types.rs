

use serde::Serialize;

use serde::Deserialize;

/// This is the data input for the teller agent 
#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct ChatMessageInput {
 
    pub api_key: Option<String>, 

    pub body: String, 

}





/// This is the data output for the teller agent 
#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct  ChatMessageOutput {
 
     

    pub body: String, 

      pub  tx_array: Option<Vec<RawTxInput>> ,

      pub  structured_data: Option< serde_json::Value  >


}



#[derive( Deserialize,  Clone,Debug, Serialize)]
 pub struct RawTxInput {
      pub chain_id: i64,
    pub to_address: String ,
    pub input_bytes:   String   ,
    pub description: Option<String>,
    pub description_short: Option<String>,

 }