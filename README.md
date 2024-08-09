# Code VM
![license][license-image]
![version][version-image]

[version-image]: https://img.shields.io/badge/version-0.1.0-blue.svg?style=flat
[license-image]: https://img.shields.io/badge/license-MIT-blue.svg?style=flat

The Code Virtual Machine (Code VM) is a program that runs on the Solana
blockchain. It is purpose built for payments and reduces transaction fees by up
to `95%` and account rent by `80%` when compared to using non-virtualized
accounts. Additionally, the Code VM can compress dormant accounts off-chain,
reducing account rent to effectively `zero`.  Compressed accounts can be
decompressed when needed — either automatically using the Code app or manually
through a public indexer. 

The Code VM is designed to be used by the Code app, to deliver a seamless
payments experience to millions of users at negligible cost.

##  What is Code?

[Code](https://getcode.com) is a mobile app that leverages self custodial 
blockchain technology to deliver a seamless payments experience that is instant, 
global, and private. 

## Audits

| Program | Mainnet | Audited By | Audit Report | Version | Commit |
| --- | --- | --- | --- | --- | --- |
| [timelock](https://github.com/code-wallet/code-program-library/tree/main/timelock) | [time2Z2...A1W2YnJ](https://explorer.solana.com/address/time2Z2SCnn3qYg3ULKVtdkh8YmZ5jFdKicnA1W2YnJ) | OtterSec | [Completed](https://github.com/code-wallet/code-program-library/tree/main/security/audits/getcode_timelock_audit_final.pdf) | v0.1.0 | [3d78dcf](https://github.com/code-wallet/code-program-library/commit/3d78dcf2201cdc047aed7a247e59522a5198e1a8) |
| [splitter](https://github.com/code-wallet/code-program-library/tree/main/splitter) | [spLit2e...cUjwpwW](https://explorer.solana.com/address/spLit2eb13Tz93if6aJM136nUWki5PVUsoEjcUjwpwW) | OtterSec | [Completed](https://github.com/code-wallet/code-program-library/tree/main/security/audits/getcode_splitter_audit_final.pdf) | v0.1.0 | [3d78dcf](https://github.com/code-wallet/code-program-library/commit/3d78dcf2201cdc047aed7a247e59522a5198e1a8) |
|[code_vm](https://github.com/code-payments/code-vm/tree/main/programs/code-vm/src) | tbd | OtterSec | pre-audit phase | tbd | tbd |

## Release Schedule

We're currently working towards a mainnet release and are looking for feedback
from the community. Please reach out to us on [Discord](https://discord.gg/T8Tpj8DBFp) or [Twitter](https://twitter.com/getcode) if you have any
questions or feedback.

<br>

| Milestone | Status | Version | Date |
| --- | --- | --- | --- |
| Preview Release | Completed | v0.1.0 | Aug 9th, 2024 |
| Golang/JS Clients | pending | - | - |
| Indexer Service | pending | - | - |
| Code VM Explorer | - | - | - |
| Sequencer Integration | pending | - | - |
| Mobile App Integration | - | - | - |
| Documentation | - | - | - |
| Audit | - | - | - |
| Bugfix Release | - | - | - |
| Devnet | tbd | - | - |
| Testnet | tbd | - | - |
| Mainnet-beta | tbd | v1.0.0 | tbd |



## Quick Start

1. Install Solana CLI: https://docs.solana.com/de/cli/install-solana-cli-tools
2. Open Terminal: solana-test-validator to start a local validator
3. In vscode in the terminal run: "npm install" to install the node packages needed to run the tests.
4. Then: anchor build && anchor deploy 
5. Copy the deployed program id from the terminal and paste it into the lib.rs and the anchor.toml file
6. Then: anchor test or anchor test --skip-local-validator depending on your node version

Maybe you also need to install Anchor or Rust: https://www.anchor-lang.com/docs/installation

## Versions

Please make sure you have the following versions installed:

```bash
% anchor --version
anchor-cli 0.30.0

% rustc --version
rustc 1.76.0 (07dca489a 2024-02-04)

% solana --version
solana-cli 1.18.9 (src:9a7dd9ca; feat:3469865029, client:SolanaLabs)
```

## Getting Help

If you have any questions or need help, please reach out to us on [Discord](https://discord.gg/T8Tpj8DBFp) or [Twitter](https://twitter.com/getcode).

## Security and Issue Disclosures

In the interest of protecting the security of our users and their funds, we ask
that if you discover any security vulnerabilities please report them using this
[Report a Vulnerability](https://github.com/code-wallet/code-program-library/security/advisories/new)
link.
