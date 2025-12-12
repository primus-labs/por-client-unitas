
## Overview

Here provides a client that interacts with Primus Network and zkVM program. It supports uploading zkVM programs, executing proof requests, etc.. Docker builds are also supported for deployment.

## Build

```sh
npm install
```

## Quick Start

1. **Copy the environment file**
    Copy `.env.example` to `.env`.

2. **Set authentication parameters**
    Contact **Primus Labs** to obtain your `TOKEN` and `PROJECT_ID`, then set them in `.env`.

3. **Configure Binance API keys**
    Provide at least one Binance API key pair:

    ```env
    BINANCE_API_KEY1=...
    BINANCE_API_SECRET1=...
    ```

    You may add additional pairs (`BINANCE_API_KEY2`, `BINANCE_API_SECRET2`, etc.) for multi-account support.

4. **Configure blockchain settings (optional)**
    Adjust `RPC_URL` as needed. Defaults target **Base Mainnet**:

    | CHAIN_ID | RPC_URL                  | Chain        |
    | -------- | ------------------------ | ------------ |
    | 8453     | https://mainnet.base.org | Base Mainnet |
    | 84532    | https://sepolia.base.org | Base Sepolia |
 
5. **Set zkTLS mode (optional)**
    The default `ZKTLS_MODE` is `POR`. If you want to change the mode to `DVC`, please also set your `PRIVATE_KEY`.

6. **Upload the zkVM program**
    (Refer to the [program README](../program/README.md) for compilation instructions.)

    ```sh
    npx por-cli uploadProgram --filepath ../program/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zktls-program
    ```

   After uploading, set the returned program ID as `PROGRAM_ID` in your `.env`.

7. **Run the client**

    ```sh
    npx tsx src/binance.ts 
    ```

## Docker Build and Run

**Build the Docker image**

```sh
sudo docker build -t primuslabs/por-client-unitas:v0.1.0 .
```

**Run the container**

```sh
docker run --rm --env-file .env primuslabs/por-client-unitas:v0.1.3
```

**Or use Docker Compose**

```sh
docker compose up
```


## Configuration

The client relies on environment variables for authentication, program execution, Binance API access, and optional blockchain integration.

Below is a complete description of all fields defined in `.env.example`.


## 1. **User Configuration (Required)**

| Variable     | Required   | Description                                      |
| ------------ | ---------- | ------------------------------------------------ |
| `TOKEN`      | ✔ Required | Authentication token assigned by Primus Labs.    |
| `PROJECT_ID` | ✔ Required | Project identifier associated with your account. |


## 2. **Program Configuration (Required)**

| Variable     | Required   | Description                                               |
| ------------ | ---------- | --------------------------------------------------------- |
| `PROGRAM_ID` | ✔ Required | Program ID returned after uploading your zkVM executable. |


## 3. **Binance API Configuration**



| Variable                | Required   | Description                                                         |
| ----------------------- | ---------- | ------------------------------------------------------------------- |
| `BINANCE_API_KEY{i}`    | ✔ Required | API key for Binance account.                                        |
| `BINANCE_API_SECRET{i}` | ✔ Required | API secret for Binance account.                                     |
| `BINANCE_RECV_WINDOW`   | Optional   | Receive window in seconds for Binance API requests (default: `60`). |

* At least **one** `(API_KEY, API_SECRET)` pair must be provided.


## 4. **Other Settings (Optional)**

| Variable           | Required | Description                                                        |
| ------------------ | -------- | ------------------------------------------------------------------ |
| `INTERVAL`         | Optional | Worker execution interval (seconds). Default: `1800` (30 minutes). |
| `LOG_VERBOSE`      | Optional | Logging verbosity (`0` = off; higher numbers = more detail).       |
| `ZKVM_SERVICE_URL` | Optional | Endpoint of the zkVM service.                                      |


## 5. **Blockchain Configuration (Optional)**


| Variable      | Required | Description                                |
| ------------- | -------- | ------------------------------------------ |
| `RPC_URL`     | Optional | RPC endpoint for the selected chain.       |
| `PRIVATE_KEY` | Optional | Wallet private key for signing operations. |


## 6. **zkTLS Mode**

| Variable     | Required | Description                           |
| ------------ | -------- | ------------------------------------- |
| `ZKTLS_MODE` | Optional | zkTLS mode: `POR` (default) or `DVC`. |

* If `ZKTLS_MODE=DVC`, a valid `PRIVATE_KEY` **must** be provided.

