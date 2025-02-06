# Code VM
![license][license-image]
![version][version-image]

[version-image]: https://img.shields.io/badge/version-0.2.0-blue.svg?style=flat
[license-image]: https://img.shields.io/badge/license-MIT-blue.svg?style=flat

The Code Virtual Machine (Code VM) is a program that runs on the Solana
blockchain. It is purpose built for payments and reduces transaction fees by up
to `95%` and account rent by `80%` when compared to using non-virtualized
accounts. The VM operates on compact representations of accounts but can further 
compress dormant accounts off-chain, effectively reducing rent to `zero`. 
Compressed accounts can be decompressed when needed — either automatically 
using the Code app or manually through a public indexer.

The Code VM is designed to be used by the Code app, to deliver a seamless
payments experience to millions of users at negligible cost.

##  What is Code?

[Code](https://getcode.com) is a mobile app that leverages self custodial 
blockchain technology to deliver a seamless payments experience that is instant, 
global, and private. 

## Deployments

| Network | Address |
| --- | --- |
| Mainnet | [vmZ1WU...5YMRTJ](https://explorer.solana.com/address/vmZ1WUq8SxjBWcaeTCvgJRZbS84R61uniFsQy5YMRTJ) |
| Devnet | [J8FLfS...Rr3L24](https://explorer.solana.com/address/J8FLfS8rqBcQ3hH8KTfQF3zBNG3r3uaG2WqfNoRr3L24?cluster=devnet) |

## Audits

| Audited By | Status | Audit Report | Version | Commit |
| --- | --- | --- | --- | --- |
| OtterSec | audit phase | - | tbd | tbd |

## Release Schedule

We're currently working towards a mainnet release and are looking for feedback
from the community. Please reach out to us on [Discord](https://discord.gg/T8Tpj8DBFp) or [Twitter](https://twitter.com/getcode) if you have any
questions or feedback.

<br>

| Milestone | Status | Code | Flipchat |
| --- | --- | --- | --- |
| Preview Release | Released | - | Aug 9th, 2024 |
| Optimized Release | Released | - | Oct 24th, 2024 |
| Audited Release | WIP | - | - |
| IDLs | [Released](https://github.com/code-payments/code-vm/blob/main/idl/code_vm.json) | - | Oct 30th, 2024 |
| Indexer Service | [Released](https://github.com/code-payments/code-vm-indexer) | - | Aug 15th, 2024 |
| Sequencer Integration | Released | - | November 25th, 2024 |
| Mobile App Integration | Released | - | November 27th, 2024 |
| Code VM Explorer | - | - | - |
| Documentation | - | - | - |

## Quick Start

1. Install Solana CLI: https://docs.solana.com/de/cli/install-solana-cli-tools
2. Open Terminal: `cargo build-sbf & cargo test-sbf -- --nocapture`


## Versions

Please make sure you have the following versions installed:

```bash
% rustc --version
rustc 1.76.0 (07dca489a 2024-02-04)

% solana --version
solana-cli 1.18.9 (src:9a7dd9ca; feat:3469865029, client:SolanaLabs)
```

## Getting Help

If you have any questions or need help, please reach out to us on [Discord](https://discord.gg/T8Tpj8DBFp) or [Twitter](https://twitter.com/getcode).

## Community Feedback & Contributions
While we can't guarantee that all feedback will be implemented, we are always 
open to hearing from the community. If you have any suggestions or feedback,
please reach out to us.

## Security and Issue Disclosures

In the interest of protecting the security of our users and their funds, we ask
that if you discover any security vulnerabilities please report them using this
[Report a Vulnerability](https://github.com/code-wallet/code-program-library/security/advisories/new)
link.
