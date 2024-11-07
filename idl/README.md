# Code VM IDL
This is not the actual program, it is a dummy program that can be used to 
generate the IDL for the [real](https://github.com/code-payments/code-vm/tree/main/program) Code VM. 

## What is an IDL?
An IDL can be used to generate clients in various languages and is used by
solana explorers to display the program's API.

At the moment, developers can either use `shank` or `anchor` to create IDLs.
Anchor is the most popular framework, so we're using it to generate the IDL for
the Code VM. Using `shank` would have required marking up our actual program
code, which we didn't want to do.

Currently, there is no standard way to generate IDLs for Solana programs. See
[RFC-00008](https://forum.solana.com/t/srfc-00008-idl-standard/66) for more
information.

## Requirements

- Anchor 0.31.1 (for building the dummy program)
- Bun.js (for running the discriminator update script)
- Vitest (for testing the IDL)

## Quick Start

To generate the IDL, run the following command after you've installed the npm
dependencies:

```bash
make idl
```

You'll find the generated IDL in the `target/idl` directory. Note that you'll
see some errors thrown by Anchor, but you can ignore them.

## Using the IDL

Unlike with a standard Anchor program, you can't use `anchor test` to test the
IDL. It would automatically replace our correct IDL with one that has different
discriminators. See the `Makefile` for more context.

Instead, you'll need to manually run a `solana-test-validator` and create a
`keypair-owner.json` file.

Once you have those, you can run a test script using the following command:
    
```bash
vitest run ./tests/vm.test.ts --testTimeout 25000 --bail 1
```

## ImHex Pattern

In addition to the IDL, we're also using the ImHex pattern to allow you to
decode accounts. Currently, the explorers don't support single byte
discriminators, so we're using the ImHex pattern to decode the accounts while
testing. You can learn more about ImHex [here](https://imhex.werwolv.net/).
