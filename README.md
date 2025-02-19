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

> [!NOTE]
> The Code VM is designed specifically to be used by the [Code](https://getcode.com) and [Flipchat](https://flipchat.xyz/) mobile apps, prioritizing **seamless payments**. As a result, this codebase is not intended as a generalized foundation for other projects.

## Audits

| Network | Address | Audited By | Status | Audit Report | Commit |
| --- | --- | --- | --- | --- | --- |
| Mainnet | [vmZ1WU...5YMRTJ](https://explorer.solana.com/address/vmZ1WUq8SxjBWcaeTCvgJRZbS84R61uniFsQy5YMRTJ) | [OtterSec](https://osec.io/) | Completed | [Report](https://github.com/code-payments/code-vm/blob/main/docs/code_audit_final.pdf) | [c49f95d9](https://github.com/code-payments/code-vm/commit/c49f95d92095eb5955955f1bc888e25957b2f64d) |
| Devnet | [J8FLfS...Rr3L24](https://explorer.solana.com/address/J8FLfS8rqBcQ3hH8KTfQF3zBNG3r3uaG2WqfNoRr3L24?cluster=devnet) | - | - | - | - |

## Dev Milestones

:white_check_mark: The on-chain program itself is largely completed.

:white_check_mark: Flipchat is using the program in mainnet.

:white_check_mark: OtterSec has completed an audit of the program.

We’re currently integrating the VM into a fork of the Code app called Flipchat. 
This allows us to make changes to backend services more efficiently without 
disrupting Code app users. 

A key obejctive for us has been that the Code mobile app could be migrated
to the VM with minimal effort. From the app’s perspective, it should behave
as if it’s using standard Solana accounts. All the VM complexity should be
abstracted away by the backend services.

We’re happy to report that this key objective has been achieved on Flipchat — 
our mobile app team migrated payments using only a handful of minor adjustments 
to the original transaction mechanics. To enable this from the VM side, we made 
intentional trade-offs that optimize for core objectives instead of on-chain 
compute units.

#### Release Schedule

| Milestone | Status | VM | Flipchat | Code |
| --- | --- | --- | --- | --- |
| Anchor Version | Done |Aug 9th, 2024 | - | - |
| Steel Version | Done | Oct 24th, 2024 | - | - |
| Audited Release | Done | Feb 18th, 2024 | - | - |
| IDLs | [Done](https://github.com/code-payments/code-vm/blob/main/idl/code_vm.json) | Oct 30th, 2024 | - | - |
| Indexer | [Done](https://github.com/code-payments/code-vm-indexer) | - | Aug 15th, 2024 | tbd |
| Sequencer | Done | - | November 25th, 2024 | tbd |
| Explorer | tbd |  | |  |
| Documentation | tbd |  |  | |


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

