### Overview
The Cardano Smart Contract scheme pushes a lot of the code off-chain.
Naumachia is designed to make the development of off-chain code as easy as possible, but also give you an
environment to test your on-chain code.

Included in the library are the tools for declaratively orchestrating interactions with validator scripts,
minting policies, and wallets;
building and checking your transaction against your on-chain code;
testing all of this code at multiple abstraction layers;
deploying, managing, and interacting with your Smart Contract in production.

Intended to be used as the off-chain backend for [Aiken][1]
or any other on-chain script (UPLC) source :)

Naumachia is meant as an alternative for the Plutus Application Backend (PAB).

Checkout the feature progress [roadmap](docs/ROADMAP.md)

#### Goals
- Make Cardano Smart Contracts easy
- Help Smart Contract developers prototype in minutes
- Make [TDD][2] a priority for SC development
    - Enable Unit Tests for your Plutus/Aiken/Helios/Raw UPLC Scripts using the [Aiken][1] CEK Machine
    - Enable Unit Tests for your entire Smart Contract with mocked backends
    - Give a clean interface for external parties to write against
- Provide adaptors for deploying and interacting with your live Smart Contract in production
- Trireme will be a CLI tool for devs and end-users alike to manage their keys, secrets, and dApps.
#### Long-term Goals
- Allow your Smart Contract to be compiled into WASM and injected into your web dApp
    - Provide adaptors for interacting with browser wallets and your chosen external services
- Auto generate simple UIs, e.g. CLIs, web interfaces, etc