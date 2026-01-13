## Quick Start

### 1. Copy the configuration file

Copy the example configuration file and rename it to `.config.yml`:

```sh
cp .config.example.yml .config.yml
```

### 2. Set authentication parameters

Contact **Primus Labs** to obtain your:

* `userToken`
* `projectId`

Then configure them under `app.identity` in `.config.yml`:

```yml
app:
  identity:
    userToken: "<YOUR_TOKEN>"
    projectId: "<YOUR_PROJECT_ID>"
```

### 3. Configure data source API keys

Configure one or more API key pairs for the supported data source. At least **one data source** must be configured, and you may configure **multiple data sources** simultaneously.

Example (Binance):

```yml
datasource: # at least one of: binance, aster
  binance:
    - apiKey: "binance-key-123"
      apiSecret: "binance-secret-abc"
      kind: ["spot", "usds-futures"]
```

### 4. Configure private key

```yml
app:
  blockchain:
    signer:
      privateKey: "<PRIVATE_KEY>"
```

### 5. Run the client

Start the client using Docker Compose:

```sh
docker compose up -d
```

**Notes**: If you update the configuration (for example, adding, modifying, or removing API key pairs), simply edit `.config.yml`. The changes will automatically take effect **in the next execution loop**, without restarting the container.
