# por-client-unitas

## Overview

Please refer to https://github.com/primus-labs/por-demo.

## Run

Refer to [client](./client/README.md).

## Supported Exchanges and Account Types

The system currently supports the following exchanges and account categories:

### Binance

* **Unified Account**
  `https://papi.binance.com/papi/v1/balance`
  [API DOC](https://developers.binance.com/docs/derivatives/portfolio-margin/account)
* **Spot Account**
  `https://api.binance.com/api/v3/account`
  [API DOC](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints#account-information-user_data)
* **Futures Account (USDⓈ-M)**
  `https://fapi.binance.com/fapi/v3/balance`
  [API DOC](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Futures-Account-Balance-V3)
* **Margin Account**
  `https://api.binance.com/sapi/v1/margin/isolated/account`
  [API DOC](https://developers.binance.com/docs/margin_trading/account/Query-Isolated-Margin-Account-Info)

### Aster

* **Spot Account**
  `https://sapi.asterdex.com/api/v1/account`
  [API DOC](https://github.com/asterdex/api-docs/blob/master/aster-finance-spot-api.md#account-information-user_data)
* **Futures Account**
  `https://fapi.asterdex.com/fapi/v2/balance`
  [API DOC](https://github.com/asterdex/api-docs/blob/master/aster-finance-futures-api.md#futures-account-balance-v2-user_data)
