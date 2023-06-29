use futures::future::join_all;
use std::fs;
use web3::types::Bytes;
use ethereum_tx_sign::LegacyTransaction;
use ethereum_tx_sign::Transaction;


#[tokio::main]
async fn main() -> web3::Result<()> {
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

    // Get the private key to use
    let private_key = json
        .get("privateKey")
        .expect("Config should have 'privateKey' key");

    // Get the amount of transaction to send
    let tx_amount = json
        .get("txAmount")
        .expect("Config should have 'txAmount' key");

    // Get the RPC url
    let rpc_url = json.get("rpcUrl").expect("Config should have 'rpcUrl' key");

    let transport = web3::transports::Http::new(rpc_url.as_str().unwrap())?;
    let web3 = web3::Web3::new(transport);

    // Craft the transaction
    let mut decoded_calldata = [0; 228]; // length of a swapExactETHForTokens calldatas
    hex::decode_to_slice(calldata.as_str().unwrap(), &mut decoded_calldata).expect("Decoding failed");

    let mut decoded_to = [0; 20]; // length of a swapExactETHForTokens calldatas
    hex::decode_to_slice(to.as_str().unwrap(), &mut decoded_to).expect("Decoding failed");

    let mut decoded_private_key = [0; 32]; // length of a swapExactETHForTokens calldatas
    hex::decode_to_slice(private_key.as_str().unwrap(), &mut decoded_private_key).expect("Decoding failed");

    let tx = LegacyTransaction {
        chain: 1,
        nonce: 0,
        to: Some(decoded_to),
        value: u128::from(value.as_u64().unwrap()),
        gas_price: u128::from(tx_amount.as_u64().unwrap()) * 1000 + 1,
        gas: 500000,
        data: decoded_calldata.to_vec(),
    };

    // Create a vector to hold all our pending futures.
    let mut pending_txs = Vec::new();

    for nonce in 0..tx_amount.as_u64().unwrap() {
        let mut _tx = tx.clone();
        _tx.nonce = u128::from(nonce);

        _tx.gas_price = tx.gas_price - u128::from(nonce) * 1000;

        // print gas price
        println!("Gas price: {}", _tx.gas_price);

        let ecdsa = _tx.ecdsa(&decoded_private_key).unwrap();
        let transaction_bytes = _tx.sign(&ecdsa);

        let pending_tx = web3.eth().send_raw_transaction(Bytes::from(transaction_bytes));
        pending_txs.push(pending_tx);
    }

    // Wait for all transactions to be mined concurrently.
    let results = join_all(pending_txs).await;

    // Handle the results.
    for result in results {
        match result {
            Ok(tx_hash) => println!("Transaction mined with hash: {}", tx_hash),
            Err(e) => eprintln!("Error mining transaction: {}", e),
        }
    }
    
    Ok(())
}
