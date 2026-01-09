# por-client-unitas

## Overview

This project verifies asset balances across multiple exchanges and account types, with support for **simultaneous zero-knowledge proofs over multiple accounts**.

It is composed of two main components:

* **[Client](./client/README.md)**: Provides user-facing functionality to configure exchanges, generate zkTLS attestations, submit tasks, and retrieve proof results.

* **[zkVM Program](./zkvm-program/README.md)**: User-defined business logic compiled and executed inside a **zkVM** running within a **TEE**, responsible for verification, asset aggregation, and proof generation.

Together, these components enable privacy-preserving verification of exchange account balances using **zkTLS**, **TEE**, and **zero-knowledge proofs**.


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

### Aster

* **Spot Account**
  `https://sapi.asterdex.com/api/v1/account`
  [API DOC](https://github.com/asterdex/api-docs/blob/master/aster-finance-spot-api.md#account-information-user_data)
* **Futures Account**
  `https://fapi.asterdex.com/fapi/v2/balance`
  [API DOC](https://github.com/asterdex/api-docs/blob/master/aster-finance-futures-api.md#futures-account-balance-v2-user_data)


## Workflow

1. Generate zkTLS **attestations** for exchange account data via the **Primus Network**.
2. Submit the attestations to a **zkVM program** running inside a **TEE**.
3. Execute verification and business logic inside the zkVM (e.g. validate attestations, extract balances, aggregate assets).
4. Generate zero-knowledge proofs using the **Succinct Network**.
5. Return proofs and verified results to the client.


For a complete conceptual introduction, see **[DVC-Intro](https://github.com/primus-labs/DVC-Intro)**.

