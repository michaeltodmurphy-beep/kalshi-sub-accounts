# kalshi-sub-accounts

A Rust client for managing subaccounts via the [Kalshi Trading API](https://docs.kalshi.com/welcome).

## What it does

This project demonstrates how to authenticate with the Kalshi API using RSA-PSS signatures and interact with the subaccount endpoints:

- **Create a subaccount** — up to 32 subaccounts per user
- **List all subaccounts** — view every subaccount on your account
- **Get subaccount balance** — check the balance (in cents) of any subaccount
- **Transfer funds between subaccounts** — move money between subaccounts, including to/from your main account (subaccount 0)

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, edition 2021)
- A Kalshi API key
- An RSA private key (PEM format) associated with your Kalshi API key

## Configuration

Set the following environment variables before running:

```bash
export KALSHI_API_KEY="your-api-key-id"
export KALSHI_PRIVATE_KEY_PATH="path/to/kalshi_private_key.pem"
```

`KALSHI_PRIVATE_KEY_PATH` defaults to `kalshi_private_key.pem` in the current directory if not set.

## Running

```bash
cargo run
```

## Notes

- **Amounts are in cents** (integer). For example, `1000` = $10.00.
- **Subaccount 0** is your main/root account.
- You can create **up to 32 subaccounts** per user.
- Authentication uses RSA-PSS (SHA-256). Each request is signed with `timestamp_ms + HTTP_METHOD + path` and the signature is sent as a hex-encoded header.

## API Reference

- [Kalshi API Documentation](https://docs.kalshi.com/welcome)
