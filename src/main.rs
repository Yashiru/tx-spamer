use ethers::{
    core::types::TransactionRequest,
    middleware::SignerMiddleware,
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer},
    types::Bytes,
};
use eyre::Result;
use futures::future::join_all;
use std::fs;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* -------------------------------------------------------------------------- */
    /*                              Read config file                              */
    /* -------------------------------------------------------------------------- */

    /* ------------------------------- Parse JSON ------------------------------- */
    let file = fs::File::open("transaction.config.json").expect("file should open read only");
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

    // Get the signer private key
    let private_key = json
        .get("privateKey")
        .expect("Config should have 'privateKey' key");

    // Get the RPC url
    let rpc_url = json.get("rpcUrl").expect("Config should have 'rpcUrl' key");

    /* -------------------------------------------------------------------------- */
    /*                               Setup provider                               */
    /* -------------------------------------------------------------------------- */

    // Create a wallet from a private key
    let wallet: LocalWallet = private_key
        .as_str()
        .unwrap()
        .parse::<LocalWallet>()
        .unwrap();

    // Connect to the network
    let provider = Provider::<Ws>::connect(rpc_url.as_str().unwrap()).await?;

    // Connect the wallet to the provider
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(1 as u64));

    /* -------------------------------------------------------------------------- */
    /*                              Send transactions                             */
    /* -------------------------------------------------------------------------- */

    // Craft the transaction
    let mut decoded = [0; 228]; // length of a swapExactETHForTokens calldatas
    hex::decode_to_slice(calldata.as_str().unwrap(), &mut decoded).expect("Decoding failed");
    let tx = TransactionRequest::new()
        .to(to.as_str().unwrap())
        .data(Bytes::from(decoded))
        .value(value.as_u64().unwrap());

    // Create a vector to hold all our pending futures.
    let mut pending_txs = Vec::new();

    // Send the transactions
    for nonce in 0..tx_amount.as_u64().unwrap() {
        let pendingTx = client.send_transaction(tx.clone().nonce(nonce), None);
        pending_txs.push(pendingTx);
    }

    println!("Sending {} transactions", tx_amount.as_u64().unwrap());

    // Wait for all transactions to be mined concurrently.
    let results = join_all(pending_txs).await;

    // Handle the results.
    for result in results {
        match result {
            Ok(tx) => println!("Transaction mined with hash: {}", tx.tx_hash()),
            Err(e) => eprintln!("Error mining transaction: {}", e),
        }
    }

    Ok(())
}
