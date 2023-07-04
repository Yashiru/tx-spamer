use futures::future::join_all;
use std::fs;
use web3::transports::WebSocket;
use web3::{
    types::{Address, TransactionRequest, U256},
    Transport, Web3,
};

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

    // Get the amount of transaction to send per block
    let tx_per_block = json
        .get("txPerBlock")
        .expect("Config should have 'txPerBlock' key");

    // Get the the amount of time to wait between each block
    let block_mining_ms_pause = json
        .get("blockMiningMsPause")
        .expect("Config should have 'blockMiningMsPause' key");

    // Get the RPC url
    let rpc_url = json.get("rpcUrl").expect("Config should have 'rpcUrl' key");

    /* -------------------------------------------------------------------------- */
    /*                            Start script message                            */
    /* -------------------------------------------------------------------------- */

    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!(
        "   Sending \x1b[35m{} transactions\x1b[0m with \x1b[36m{} tx per blocks\x1b[0m\n   With \x1b[33m{} ms\x1b[0m additonal pause between each blocks.",
        tx_amount,
        tx_per_block,
        block_mining_ms_pause
    );
    println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

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

    web3.transport()
        .execute(
            "anvil_setNonce",
            vec![serde_json::to_value(&my_account)?, serde_json::to_value(0)?],
        )
        .await?;
    println!("\x1b[32m✔\x1b[0m Nonce set to 0.");

    let mut succeeded_txs = 0;
    let mut failed_txs = 0;

    for nonce in 0..tx_amount.as_u64().unwrap() {
        if nonce % tx_per_block.as_u64().unwrap() == 0 && nonce != 0 {
            let result =
                mine_and_wait(&web3, pending_txs, block_mining_ms_pause.as_u64().unwrap()).await;

            match result {
                Ok((succeeded, failed)) => {
                    succeeded_txs += succeeded;
                    failed_txs += failed;
                }
                Err(_) => println!("Error while mining block"),
            }

            pending_txs = Vec::new();
        }

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

    let result = mine_and_wait(&web3, pending_txs, block_mining_ms_pause.as_u64().unwrap()).await;

    match result {
        Ok((succeeded, failed)) => {
            succeeded_txs += succeeded;
            failed_txs += failed;
        }
        Err(_) => println!("Error while mining block"),
    }

    if succeeded_txs > 0 {
        println!(
            "\n\x1b[32m✔\x1b[0m \x1b[35m{} transactions\x1b[0m has been sent in \x1b[36m{} blocks\x1b[0m with \x1b[33m{} ms\x1b[0m pause between each blocks.",
            succeeded_txs,
            tx_amount.as_u64().unwrap() / tx_per_block.as_u64().unwrap(),
            block_mining_ms_pause.as_u64().unwrap()
        );
    }

    if failed_txs > 0 {
        println!("\x1b[31m✖\x1b[0m {} failed transactions (Mempool probably full, try empty it)", failed_txs);
    }

    Ok(())
}

async fn mine_and_wait(
    web3: &Web3<WebSocket>,
    pending_txs: Vec<
        web3::helpers::CallFuture<
            web3::types::H256,
            web3::transports::ws::Response<
                serde_json::Value,
                fn(
                    Result<Vec<Result<serde_json::Value, web3::Error>>, web3::Error>,
                ) -> Result<serde_json::Value, web3::Error>,
            >,
        >,
    >,
    pause_ms: u64,
) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let mut succeeded_txs = 0;
    let mut failed_txs = 0;

    let results = join_all(pending_txs).await;

    for result in results {
        match result {
            Ok(_) => succeeded_txs += 1,
            Err(_) => failed_txs += 1,
        }
    }

    let result = web3
        .transport()
        .execute("anvil_mine", vec![serde_json::to_value(1)?])
        .await;

    match result {
        Ok(_) => println!("👉 \x1b[32mBlock mined\x1b[0m with {} transactions", succeeded_txs),
        Err(_) => println!("👉 \x1b[32mBlock mining failed\\x1b[31m"),
    }

    // web3.transport().execute("anvil_dropTransaction", vec![]).await?;
    // Sleep for the specified amount of time
    std::thread::sleep(std::time::Duration::from_millis(pause_ms + 500));

    Ok((succeeded_txs, failed_txs))
}
