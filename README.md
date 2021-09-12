# SPL token transaction speed benchmark

This benchmark logs transaction duration statistics to standard output. The benchmark waits
transactions to reach the `Confirmed` commitment level.

## Usage

`cargo run -- -i <keypair.json> -n <num_test_keypairs> -u <rpc_url>`
