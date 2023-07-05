<img align="right" width="150" height="150" top="100" src="https://avatars.githubusercontent.com/u/5430905?s=200&v=4">

# Transaction spammer

**A Rust-based tool for spamming a transaction.**

## Objective

This tool is designed to streamline the process of quickly dispatching a substantial number of transactions to a local Anvil Ethereum node. It serves as a valuable asset for testing event capture capabilities within a backend infrastructure for example.

## Setup

### Anvil
You must run a Anvil node as follow:

```bash
anvil \
  --accounts 1 \
  --balance 100000000 \
  --fork-url <YOUR_FORK_URL> \
  --order FIFO \
  --gas-price 1 \
  --no-mining
```

### Script configuration
You can find the script configuration in `config.json`
```javascript
{
  // The total amount of transactions to send
  txAmount: 5000,

  // The transaction amount per mined block 
  txPerBlock: 1000,

  // An additional pause between each block (Keep in mind that the block mining itself take some time)
  blockMiningMsPause: 0,
  
  // The WebSocket RPC url to send the transactions to
  rpcUrl: "ws://127.0.0.1:8545" 
  
  // The transactions to be sent
  transactions: [
    {
      // The contract address to call (without 0x)
      to: "9a5...EA44", 

      // The call datas (without 0x)
      calldata: "afaf...1e1e",

      // The call value
      value: 10,
    },
    {
      ...
    },
    {
      ...
    }
  ]
}
```

### Run the script

**Install Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Setup up script configuration**  
You need to configure the following parameters:
   - `txAmount`: The total amount of transactions you wish to send.

   - `txPerBlock`: The amount of transactions per mined block.

   - `blockMiningMsPause`: If you wish to have an additional pause after each block mining, specify it here (in milliseconds). Keep in mind that the block mining process itself takes some time.

   - `rpcUrl`: This is the WebSocket RPC URL to which the transactions will be sent.

   - `transactions`: Define the transactions you want to send in this array. For each transaction, provide the contract address (without 0x) in the `to` field, the call data (without 0x) in the `calldata` field, and the call value in the `value` field.

Example:

```javascript
{
  "txAmount": 5000,
  "txPerBlock": 1000,
  "blockMiningMsPause": 0,
  "rpcUrl": "ws://127.0.0.1:8545",
  "transactions": [
    {
      "to": "9a5...EA44",
      "calldata": "afaf...1e1e",
      "value": 10
    }
  ]
}
```

> **Note**
> The program processes transactions based on the transactions array in the config.json file. It starts from the first transaction in the array, sending them one by one to the Anvil node in their listed order. The loop continues until all transactions have been dispatched according to the `txAmount` field.


**Run script with cargo**
```bash
cargo run
```

## Disclaimer

This project primarily serves as an educational platform to experiment with Rust. While its current implementation is functional, it's quite possible that improvements could be made. Suggestions and enhancements are encouraged to help me further develop my expertise in Rust.