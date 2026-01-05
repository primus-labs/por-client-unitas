
## Overview

Here provides a client that interacts with Primus Network and zkVM program. Docker builds are also supported for deployment.

## Build

```sh
npm install
```

## Quick Start

### 1. Copy the configuration file

Copy the example configuration file and rename it to `.config.yml`:

```sh
cp .config.example.yml .config.yml
```

### 2. Set authentication parameters

Contact **Primus Labs** to obtain your:

* `token`
* `projectId`
* `programId`

Then configure them under `app.identity` in `.config.yml`:

```yml
app:
  identity:
    token: "<YOUR_TOKEN>"
    projectId: "<YOUR_PROJECT_ID>"
    programId: "<YOUR_PROGRAM_ID>"
```

### 3. Configure exchange API keys

Configure one or more API key pairs for the supported exchanges.
At least **one exchange** must be configured, and you may configure **multiple exchanges** simultaneously.

Example (Binance):

```yml
exchanges: # at least one of: binance, aster
  binance:
    - apiKey: "binance-key-123"
      apiSecret: "binance-secret-abc"
      kind: ["spot", "usds-futures"]
```

### 4. Configure blockchain settings (optional)


Adjust the blockchain network via `app.blockchain.network`. The default network is **Base Mainnet (`base`)**.

You may also specify a custom RPC endpoint using `app.blockchain.rpcUrl`.

| CHAIN_ID | RPC URL                  | Network      |
| -------: | ------------------------ | ------------ |
|     8453 | https://mainnet.base.org | Base Mainnet |
|    84532 | https://sepolia.base.org | Base Sepolia |

Example:

```yml
app:
  blockchain:
    network: base-sepolia
    rpcUrl: https://sepolia.base.org
```

### 5. Configure private key (optional)

If the subscription type of the `projectId` is **`PLAN_SELF_PAID`**, you must configure a signer private key:

```yml
app:
  blockchain:
    signer:
      privateKey: "<PRIVATE_KEY>"
```


### 6. Run the client

Start the client using Docker Compose:

```sh
docker compose up
```

**Notes**: If you update the configuration (for example, adding, modifying, or removing API key pairs), simply edit `.config.yml`. The changes will automatically take effect **in the next execution loop**, without restarting the container.


## Configuration

Also see por-client-sdk.

### Structure Overview

```yaml
app:               # Core application configuration
  identity:        # Application identity and authorization info
  runtime:         # Runtime environment configuration
  services:        # External service endpoints
  blockchain:      # Blockchain connection and signer info

exchanges:         # Exchange account configurations
  binance:         # Binance exchange accounts
  aster:           # Aster exchange accounts
```


### Application Configuration (`app`)

#### 1. Identity (`app.identity`)

Contains application authorization and identification.

| Field     | Type   | Description                                      | Example           |
| --------- | ------ | ------------------------------------------------ | ----------------- |
| token     | string | Authentication token issued for this application | `"my-auth-token"` |
| projectId | string | Unique project identifier                        | `"project-123"`   |
| programId | string | Unique program identifier                        | `"program-abc"`   |

#### 2. Runtime (`app.runtime`)

Specifies the runtime environment and logging.

| Field      | Type    | Default        | Description                                            | Example        |
| ---------- | ------- | -------------- | ------------------------------------------------------ | -------------- |
| version    | string  | -              | Application version in semantic version format (x.y.z) | `"1.0.0"`      |
| env        | string  | `"production"` | Runtime environment (`development` or `production`)    | `"production"` |
| logVerbose | integer | `0`            | Log verbosity (0 = off, higher = more detail)          | `3`            |

#### 3. Services (`app.services`)

External service endpoints used by the application.

##### zkVM Service (`app.services.zkvm`)

| Field | Type   | Description               | Example                      |
| ----- | ------ | ------------------------- | ---------------------------- |
| url   | string | zkVM service endpoint URL | `"https://zkvm.example.com"` |

##### Data Service (`app.services.data`)

| Field | Type   | Description               | Example                      |
| ----- | ------ | ------------------------- | ---------------------------- |
| url   | string | Data service endpoint URL | `"https://data.example.com"` |

#### 4. Blockchain (`app.blockchain`)

Blockchain connection and signer configuration.

| Field   | Type   | Default  | Description                                                                  | Example                     |
| ------- | ------ | -------- | ---------------------------------------------------------------------------- | --------------------------- |
| network | string | `"base"` | Target blockchain network (`base` or `base-sepolia`)                         | `"base-sepolia"`            |
| rpcUrl  | string | -        | Optional custom RPC URL. If not set, default RPC for the network is used     | `"https://rpc.example.com"` |
| signer  | object | -        | Transaction signer info (required if `subscriptionType` is `PLAN_SELF_PAID`) | -                           |

##### Signer (`app.blockchain.signer`)

| Field      | Type   | Description                                 | Example            |
| ---------- | ------ | ------------------------------------------- | ------------------ |
| privateKey | string | Private key to sign blockchain transactions | `"0xabcdef123..."` |

---

### Exchange Accounts (`exchanges`)

Supports multiple exchange accounts. At least one exchange is required. Now only support Binance and Aster.

#### Fields of `exchange`

| Field       | Type    | Description                                    | Example                     |
| ----------- | ------- | ---------------------------------------------- | --------------------------- |
| apiKey      | string  | API key used to authenticate with Binance      | `"binance-key-123"`         |
| apiSecret   | string  | API secret corresponding to the API key        | `"binance-secret-abc"`      |
| enabled     | boolean | Whether this account is active (default: true) | `true`                      |
| description | string  | Optional description for this account          | `"My Binance spot account"` |
| kind        | array   | Supported Binance account types                | `["spot","usds-futures"]`   |

