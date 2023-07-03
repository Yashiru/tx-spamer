<img align="right" width="150" height="150" top="100" src="https://avatars.githubusercontent.com/u/5430905?s=200&v=4">

# Transaction spammer

**A Rust-based tool for spamming a transaction.**

## Objective

This tool is designed to facilitate the rapid dispatch of a high volume of identical transactions to an Anvil local Ethereum node. It can be used to test the event catch of a backend infrastructure, for example.

## Setup

### Anvil
You must run a Anvil node as follow:

```bash
anvil \
  --accounts 1 \
  --balance 100000000 \
  --fork-url <YOUR_FORK_URL> \
  --order FIFO \
  --gas-price 1
```

### Script configuration
You can find the script configuration in `config.json`
```json
{
  "to": "9a5...EA44", // The contract to call
  "calldata": "afaf...1e1e", // The call datas
  "value": 10, // The call value
  "txAmount": 5000, // The amount of transaction to send
  "rpcUrl": "ws://127.0.0.1:8545" // The WebSocket RPC url to send the transactions to
}
```

## Disclaimer

This project primarily serves as an educational platform to experiment with Rust. While its current implementation is functional, it's quite possible that improvements could be made. Suggestions and enhancements are encouraged to help me further develop my expertise in Rust.
