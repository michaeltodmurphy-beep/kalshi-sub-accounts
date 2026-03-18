# kalshi-sub-accounts

An interactive CLI tool written in Rust for managing [Kalshi](https://kalshi.com) subaccounts via the [Kalshi Trading API](https://docs.kalshi.com/welcome).

## What it does

This program connects to the Kalshi API using RSA-PSS authentication and lets you manage subaccounts interactively from the command line.

- **Create a subaccount** — type `Create Sub` at the prompt to create the next subaccount (up to 32 per user)

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, edition 2021)
- A Kalshi API key
- An RSA private key (**PKCS#1 PEM format**, header `-----BEGIN RSA PRIVATE KEY-----`) associated with your Kalshi API key

## Setup

### 1. Configure your API key

Copy the example config file and fill in your API key:

```bash
cp config.json.example config.json
```

Edit `config.json`:

```json
{
  "kalshi_access_key": "your-api-key-here"
}
```

### 2. Place your private key

Copy your RSA private key (**PKCS#1 PEM format**, with the header `-----BEGIN RSA PRIVATE KEY-----`) into the project directory as:

```
kalshi-private-key.pem
```

Both `config.json` and `*.pem` files are listed in `.gitignore` and will never be committed.

## Running

```bash
cargo run
```

## Available commands

| Command       | Description                                    |
|---------------|------------------------------------------------|
| `Create Sub`  | Create the next subaccount (up to 32 allowed)  |
| `exit`        | Exit the program                               |
| `quit`        | Exit the program                               |

Commands are case-insensitive.

## Notes

- You can create **up to 32 subaccounts** per Kalshi account.
- Authentication uses RSA-PSS (SHA-256). Each request is signed with `timestamp_ms + HTTP_METHOD + path` and the signature is hex-encoded.
- Credentials are loaded from `config.json` (API key) and `kalshi-private-key.pem` (private key) — never from environment variables.

## API Reference

- [Kalshi API Documentation](https://docs.kalshi.com/welcome)

