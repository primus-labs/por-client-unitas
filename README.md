# por-client-unitas

## Overview

This project is composed of two main components: **[client](./client/README.md)** and **[program](./program/README.md)**.

* **Client**: Provides user-facing functionality to upload programs, submit tasks, and retrieve results.
* **Program**: Developed and compiled by users according to their business logic, then executed within a zkVM inside a TEE to generate proofs.

The system enables verification of asset balances for Binance **spot** and **unified accounts**, with support for **simultaneous proofs across multiple accounts**.

### Workflow

1. Generate zkTLS **attestations** for account data via the **Primus Network**.
2. Send the attestations to a **zkVM program** running inside a **TEE**.
3. Execute verification and business logic in the zkVM (e.g., validate attestations, extract balances, aggregate assets).
4. Produce zero-knowledge proofs using the **Succinct Network**.

For a complete introduction, see [DVC-Intro](https://github.com/primus-labs/DVC-Intro).

