use serde_json::Value;
use web3::{types::{Address, U256}};
use std::str::FromStr;

/// A `Config` struct represents the configuration.
pub struct Config {
    /// the transaction to send
    pub transactions: Vec<Transaction>,
    /// the amount of transactions in total
    pub tx_amount: u64,
    /// the amount of transactions per block
    pub tx_per_block: u64,
    /// the pause (in milliseconds) between mining blocks
    pub block_mining_ms_pause: u64,
    /// the url of the RPC node
    pub rpc_url: String,
}

pub struct Transaction {
    /// the target address of the transaction
    pub to: Address,
    /// the call data of the transaction
    pub calldata: Vec<u8>,
    /// the value (in wei) included in the transaction
    pub value: U256,
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
        let mut txs: Vec<Transaction> = Vec::new();
        let json_txs = json_value["transactions"]
            .as_array()
            .expect("Config should have 'to' key");

        for i in 0..json_txs.len() {
            // Decode calldata from the config file
            let calldata = hex::decode(
                json_txs[i]["calldata"]
                    .as_str()
                    .expect("Config should have 'calldata' key")
                    .to_string(),
            )
            .expect("Failed to decode calldata");

            // Decode to address from the config file
            let to = Address::from_str(
                &json_txs[i]["to"]
                    .as_str()
                    .expect("Config should have 'to' key")
                    .to_string(),
            )
            .expect("Failed to decode address");

            // Decode the value from the config file
            let value = U256::from(
                json_txs[i]["value"]
                    .as_u64()
                    .expect("Config should have 'value' key"),
            );

            // Push the decoded transaction to txs
            txs.push(Transaction {
                to: to,
                calldata: calldata,
                value: value,
            })
        }

        Self {
            transactions: txs,
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
