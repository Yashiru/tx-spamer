use serde_json::Value;

/// A `Config` struct represents the configuration.
pub struct Config {
    /// the target address of the transaction
    pub to: String,
    /// the call data of the transaction
    pub calldata: String,
    /// the value (in wei) included in the transaction
    pub value: u64,
    /// the amount of transactions in total
    pub tx_amount: u64,
    /// the amount of transactions per block
    pub tx_per_block: u64,
    /// the pause (in milliseconds) between mining blocks
    pub block_mining_ms_pause: u64,
    /// the url of the RPC node
    pub rpc_url: String,
}

impl Config {
    /// This method reads the configuration from a JSON formatted value.
    /// 
    /// # Parameters
    /// - `json_value`: A reference to a JSON object.
    /// 
    /// # Returns
    /// An instance of `Config` built from the `json_value`.
    pub fn from_json(json_value: &Value) -> Self {
        Self {
            to: json_value["to"]
                .as_str()
                .expect("Config should have 'to' key")
                .to_string(),
            calldata: json_value["calldata"]
                .as_str()
                .expect("Config should have 'calldata' key")
                .to_string(),
            value: json_value["value"]
                .as_u64()
                .expect("Config should have 'value' key"),
            tx_amount: json_value["txAmount"]
                .as_u64()
                .expect("Config should have 'txAmount' key"),
            tx_per_block: json_value["txPerBlock"]
                .as_u64()
                .expect("Config should have 'txPerBlock' key"),
            block_mining_ms_pause: json_value["blockMiningMsPause"]
                .as_u64()
                .expect("Config should have 'blockMiningMsPause' key"),
            rpc_url: json_value["rpcUrl"]
                .as_str()
                .expect("Config should have 'rpcUrl' key")
                .to_string(),
        }
    }
}