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

Note, as of this writing, there is no "standard" way to generate IDLs for Solana
programs. See [RFC-00008](https://forum.solana.com/t/srfc-00008-idl-standard/66)
for more information.

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

You'll find the generated IDL
[here](https://github.com/code-payments/code-vm/blob/main/idl/code_vm.json) in
the `target/idl` directory. Note that you'll see some errors thrown by Anchor,
but you can ignore them.

## Using the IDL

There are many ways to make use of the IDL, however, one way is to decode accounts.

You can pull the IDL directly from the program if you'd like using something like this:

```js
    const url = "https://api.mainnet-beta.solana.com";
    const idl = await anchor.Program.fetchIdl("vmZ1WUq8SxjBWcaeTCvgJRZbS84R61uniFsQy5YMRTJ", getProvider(url));
    const coder = new BorshAccountsCoder(idl);

    const rawData = Buffer.from(/* your account data here */, "base64");

    const account = idl.accounts?.find((accountType: any) =>
        (rawData as Buffer).slice(0, 8).equals(coder.accountDiscriminator(accountType.name))
    );
    const accountDef = idl.types.find((type: any) => type.name === account.name);
    const decodedAccount = coder.decode(account.name, rawData);
```

## Running tests

Unlike with a standard Anchor program, you can't use `anchor test` to test the
IDL. It would automatically replace our correct IDL with one that has different
discriminators. See the `Makefile` for more context.

Instead, you'll need to manually run a `solana-test-validator` and create a
`keypair-owner.json` file.

Once you have those, you can run a test script using the following command:
    
```bash
vitest run ./tests/vm.test.ts --testTimeout 25000 --bail 1
```

## ImHex Patterns

In addition to the IDL, we're also including two ImHex pattern files that you
can use to decode accounts and instructions inside the ImHex editor. You can
learn more about ImHex [here](https://imhex.werwolv.net/).

### Instructions

The first pattern file is for decoding instructions and opcodes. You can find the pattern
[here](https://github.com/code-payments/code-vm/blob/main/idl/code_vm.instructions.hexpat).

For example, here is a decoded [transfer](https://github.com/code-payments/code-vm/blob/main/program/src/opcode/transfer.rs):

![image](https://github.com/user-attachments/assets/111180c5-2652-4603-9d75-1c3fee3d267b)

### Accounts

We also have a pattern file for decoding accounts. You can find the pattern
[here](https://github.com/code-payments/code-vm/blob/main/idl/code_vm.accounts.hexpat).

For example, here is a decoded [memory account](https://github.com/code-payments/code-vm/blob/main/api/src/cvm/state/memory.rs#L43C12-L43C25):

![image](https://github.com/user-attachments/assets/f2184e1f-8c1f-4774-a88b-8a169e72ebff)
