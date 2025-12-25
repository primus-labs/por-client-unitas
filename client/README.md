
## Overview

Here provides a client that interacts with Primus Network and zkVM program. Docker builds are also supported for deployment.

## Build

```sh
npm install
```

## Quick Start

1. **Copy the environment file**
    Copy `.env.example` to `.env`.

2. **Set authentication parameters**
    Contact **Primus Labs** to obtain your `TOKEN`, `PROJECT_ID` and `PROGRAM_ID`, then set them in `.env`.

3. **Configure Binance API keys**
    Provide at least one Binance API key pair:

    ```env
    BINANCE_API_KEY=...
    BINANCE_API_SECRET=...
    ```

    You may add additional pairs (`BINANCE_API_KEY1`, `BINANCE_API_SECRET1`, etc.) for multi-account support.

4. **Configure blockchain settings (optional)**
    Adjust `RPC_URL` as needed. Defaults target **Base Mainnet**:

    | CHAIN_ID | RPC_URL                  | Chain        |
    | -------- | ------------------------ | ------------ |
    | 8453     | https://mainnet.base.org | Base Mainnet |
    | 84532    | https://sepolia.base.org | Base Sepolia |

5. **Run the client**

    ```sh
    docker compose up
    ```

## Configuration

The client relies on environment variables for authentication, program execution, Binance API access, and optional blockchain integration.

Below is a complete description of all fields defined in `.env.example`.


| Env Variable  | Required / Conditions | Default                    | Description                                                |
| ------------- | --------------------- | -------------------------- | ---------------------------------------------------------- |
| `TOKEN`       | Required              | `""`                       | Authentication token assigned by Primus Labs.              |
| `PROJECT_ID`  | Required              | `""`                       | Project identifier associated with your account.           |
| `PROGRAM_ID`  | Required              | `""`                       | Program identifier for zkVM execution.                     |
| `LOG_VERBOSE` | Optional              | `0`                        | Logging verbosity level (`0` = off; higher = more detail). |
| `RPC_URL`     | Optional              | `https://mainnet.base.org` | RPC endpoint for blockchain interactions.                  |



### Binance API Configuration

| Env Variable          | Required / Conditions | Default | Description                                         |
| --------------------- | --------------------- | ------- | --------------------------------------------------- |
| `BINANCE_API_KEY`     | Required              | `""`    | API key for Binance account.                        |
| `BINANCE_API_SECRET`  | Required              | `""`    | API secret for Binance account.                     |
| `BINANCE_RECV_WINDOW` | Optional              | `60`    | Receive window in seconds for Binance API requests. |

* You may add additional pairs (`BINANCE_API_KEY1`, `BINANCE_API_SECRET1`, etc.) for multi-account support.


### Others

| Env Variable | Required / Conditions | Default | Description                           |
| ------------ | --------------------- | ------- | ------------------------------------- |
| `INTERVAL`   | Optional              | `1800`  | Worker execution interval in seconds. |
