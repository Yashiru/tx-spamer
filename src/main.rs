use futures::future::join_all;
use std::fs;
use web3::{
    types::{Address, TransactionRequest, U256},
    Web3, Transport,
};
use web3::transports::WebSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* -------------------------------------------------------------------------- */
    /*                              Read config file                              */
    /* -------------------------------------------------------------------------- */

    /* ------------------------------- Parse JSON ------------------------------- */
    let file = fs::File::open("config.json").expect("file should open read only");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("file should be proper JSON");

    /* ------------------------------- Get config ------------------------------- */

    // Get the contract address to the transaction to
    let to = json.get("to").expect("Config should have 'to' key");

    // Get the calls datas
    let calldata = json
        .get("calldata")
        .expect("Config should have 'calldata' key");

    // Get the calls values
    let value = json.get("value").expect("Config should have 'value' key");

    // Get the amount of transaction to send
    let tx_amount = json
        .get("txAmount")
        .expect("Config should have 'txAmount' key");

    // Get the RPC url
    let rpc_url = json.get("rpcUrl").expect("Config should have 'rpcUrl' key");

    /* -------------------------------------------------------------------------- */
    /*                               Setup provider                               */
    /* -------------------------------------------------------------------------- */

    // Connect to the network
    let transport = WebSocket::new(rpc_url.as_str().unwrap()).await?;
    let web3 = Web3::new(transport);

    let mut accounts = web3.eth().accounts().await?;
    let my_account = match accounts.pop() {
        Some(account) => account,
        None => panic!("No accounts available"),
    };

    /* -------------------------------------------------------------------------- */
    /*                              Send transactions                             */
    /* -------------------------------------------------------------------------- */

    // Craft the transaction
    let data = hex::decode(calldata.as_str().unwrap()).expect("Decoding failed");
    let value = U256::from(value.as_u64().unwrap());
    let to = Address::from_slice(&hex::decode(to.as_str().unwrap()).expect("Decoding failed"));

    // Create a vector to hold all our pending futures.
    let mut pending_txs = Vec::new();

    web3.transport().execute("anvil_setNonce", vec![
        serde_json::to_value(&my_account)?,
        serde_json::to_value(0)?,
    ]).await?;

    // Send the transactions
    for nonce in 0..tx_amount.as_u64().unwrap() {
        let tx_request = TransactionRequest {
            from: my_account,
            to: Some(to),
            value: Some(value),
            data: Some(web3::types::Bytes(data.clone())),
            nonce: Some(U256::from(nonce)),
            ..Default::default()
        };

        let pending_tx = web3.eth().send_transaction(tx_request);
        pending_txs.push(pending_tx);
    }

    println!("Sending {} transactions...", tx_amount.as_u64().unwrap());

    // Wait for all transactions to be mined concurrently.
    let results = join_all(pending_txs).await;

    // Handle the results.
    let mut i = 0;
    for result in results {
        match result {
            Ok(tx) => println!("Transaction #{:?} mined with hash: {:?}", i, tx),
            Err(e) => eprintln!("Error mining transaction: {:?}", e),
        }
        i += 1;
    }

    Ok(())
}
