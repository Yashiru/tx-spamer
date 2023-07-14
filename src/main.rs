mod utils {
    pub mod config;
}
use colored::*;
use futures::future::join_all;
use serde_json::Value;
use std::fs::File;
use web3::transports::WebSocket;
use web3::{
    types::{TransactionRequest, U256},
    Transport, Web3,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* -------------------------------------------------------------------------- */
    /*                              Read config file                              */
    /* -------------------------------------------------------------------------- */

    let json_file = File::open("config.json").unwrap_or_else(|_| panic!("Unable to open file"));
    let json_value: Value =
        serde_json::from_reader(json_file).unwrap_or_else(|_| panic!("Invalid JSON format"));
    let config = utils::config::Config::from_json(&json_value);

    /* -------------------------------------------------------------------------- */
    /*                            Start script message                            */
    /* -------------------------------------------------------------------------- */

    let border_line =
        "╔══════════════════════════════════════════════════════════════════════════╗";
    let footer_line =
        "╚══════════════════════════════════════════════════════════════════════════╝\n";

    let transactions_info = format!(
        "  Sending {} with {}",
        format!("{} transactions", config.tx_amount).magenta(),
        format!("{} tx per blocks", config.tx_per_block).cyan()
    );

    let pause_info = format!(
        "  With {} additional pause between each blocks",
        format!("{} ms", config.block_mining_ms_pause).yellow()
    );

    println!("{}", border_line);
    println!("{}", transactions_info);
    println!("{}", pause_info);
    println!("{}", footer_line);

    /* -------------------------------------------------------------------------- */
    /*                               Setup provider                               */
    /* -------------------------------------------------------------------------- */

    // Connect to the network
    let web3 = Web3::new(WebSocket::new(config.rpc_url.as_str()).await?);
    // Get the signer
    let my_account = web3
        .eth()
        .accounts()
        .await?
        .pop()
        .expect("No accounts available");
 
    /* -------------------------------------------------------------------------- */
    /*                          Setup transactions calls                          */
    /* -------------------------------------------------------------------------- */

    // Create a vector to hold all pending futures.
    let mut pending_block_txs = Vec::new();

    // Initialize nonce as 0 for nonce "prediction"
    let parameters = vec![serde_json::to_value(&my_account)?, serde_json::to_value(0)?];
    web3.transport()
        .execute("anvil_setNonce", parameters)
        .await?;

    println!("{} Nonce set to 0.", "✔".green());

    /* -------------------------------------------------------------------------- */
    /*                              Send transactions                             */
    /* -------------------------------------------------------------------------- */

    // Declare transaction counters
    let mut succeeded_txs = 0;
    let mut failed_txs = 0;

    let mut nonce = 0;
    for _ in 0..config.tx_amount / config.transactions.len() as u64 {
        // If we have reached the tx_per_block limit, mine a block and wait block_mining_ms_pause
        if nonce % config.tx_per_block < config.transactions.len() as u64 && nonce != 0 {
            let result =
                mine_and_wait(&web3, pending_block_txs, config.block_mining_ms_pause).await;

            // Update transaction counters
            match result {
                Ok((succeeded, failed)) => {
                    succeeded_txs += succeeded;
                    failed_txs += failed;
                }
                Err(_) => println!("{}", "✖ Error while mining block".red()),
            }

            pending_block_txs = Vec::new();
        }

        for i in 0..config.transactions.len() {
            // Craft the transaction
            let tx_request = TransactionRequest {
                from: my_account,
                to: Some(config.transactions[i].to),
                value: Some(config.transactions[i].value),
                data: Some(web3::types::Bytes(config.transactions[i].calldata.clone())),
                nonce: Some(U256::from(nonce)),
                ..Default::default()
            };
    
            // Send the transaction and add it to the pending_block_txs vector
            let pending_tx = web3.eth().send_transaction(tx_request);
            pending_block_txs.push(pending_tx);
            
            // Increment nonce
            nonce += 1;
        }
    }

    // Mine the last block
    let result = mine_and_wait(&web3, pending_block_txs, config.block_mining_ms_pause).await;

    // Update transaction counters
    match result {
        Ok((succeeded, failed)) => {
            succeeded_txs += succeeded;
            failed_txs += failed;
        }
        Err(_) => println!("{}", "✖ Error while mining block".red()),
    }

    /* -------------------------------------------------------------------------- */
    /*                                   Result                                   */
    /* -------------------------------------------------------------------------- */

    // Print the succeded transaction result
    if succeeded_txs > 0 {
        println!(
            "{} {} has been sent in {} with {} pause between each blocks.",
            "✔".green(),
            format!("{} transactions", succeeded_txs).magenta(),
            format!("{} blocks", config.tx_amount / config.tx_per_block).cyan(),
            format!("{} ms", config.block_mining_ms_pause).yellow()
        );
    }

    // Print the failed transaction result
    if failed_txs > 0 {
        println!(
            "{} {} failed transactions (Mempool probably full, try empty it)",
            "✖".red(),
            failed_txs
        );
    }

    Ok(())
}

/// Asynchronously mine a block, pause for a configured duration, then return the durations of mining and pause.
///
/// # Arguments
///
/// * `web3` - Web3 instance connected via WebSocket.
/// * `pending_txs` - Vector of pending Ethereum transactions.
/// * `pause_ms` - Amount of pause time after block mining (in milliseconds).
///
/// # Return
///
/// Return a `Result` which, if Ok, contains a tuple of mining and pause durations in milliseconds (u64).
/// If an error occurs, an Error Box is returned encapsulating the specific error.
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
    // Declare and initialize transaction counters
    let mut succeeded_txs = 0;
    let mut failed_txs = 0;

    // Wait for the transactions to be indexed
    let results = join_all(pending_txs).await;

    // Count the number of successful and failed transactions
    results.iter().for_each(|result| match result {
        Ok(_) => succeeded_txs += 1,
        Err(err) => {
            println!("{} {:?}", "✖".red(), err);
            failed_txs += 1
        },
    });

    // Execute block mining
    let mining_result = web3
        .transport()
        .execute("anvil_mine", vec![serde_json::to_value(1)?])
        .await;

    // Handle mining result
    if let Ok(_) = mining_result {
        println!(
            "👉 {} with {} transactions",
            "Block mined".green(),
            succeeded_txs
        );
    } else {
        println!("👉 {}", "Block mining failed".red());
    }

    // sleep for the specified amount of time
    std::thread::sleep(std::time::Duration::from_millis(pause_ms));

    // Return the transaction counters
    Ok((succeeded_txs, failed_txs))
}
