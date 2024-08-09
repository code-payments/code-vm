import * as anchor from "@coral-xyz/anchor";
import * as beet from '@metaplex-foundation/beet'
import * as beetSolana from '@metaplex-foundation/beet-solana'
import { AccountBuffer } from "./allocator";

export class MemoryAccount {
  constructor(
    readonly _discriminator: number[],
    readonly vm: anchor.web3.PublicKey,
    readonly bump: number,
    readonly name: number[],
    readonly layout: number,
  ) { }

  static readonly MAX_NAME_LEN = 32;
  static readonly LEN =           // 74 bytes
    8 +                           // anchor (discriminator)
    32 +                          // vm
    1 +                           // bump
    1 +                           // padding
    MemoryAccount.MAX_NAME_LEN;  // name

  static readonly struct = new beet.BeetArgsStruct<MemoryAccount>(
    [
      ['_discriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
      ['vm', beetSolana.publicKey],
      ['bump', beet.u8],
      ['name', beet.uniformFixedSizeArray(beet.u8, MemoryAccount.MAX_NAME_LEN)],
      ['layout', beet.u8]
    ],
    'MemoryAccount'
  )
}

export class MemoryAccountWithData {
  constructor(
    readonly info: MemoryAccount,
    readonly data: number[],
  ) { }

  static readonly LEN =        // ~4.6Mb
    MemoryAccount.LEN +        // account
    AccountBuffer.LEN;         // data

  static readonly struct = new beet.BeetArgsStruct<MemoryAccountWithData>(
    [
      ['info', MemoryAccount.struct],
      ['data', beet.uniformFixedSizeArray(beet.u8, AccountBuffer.LEN)],
    ],
    'MemoryAccountWithData'
  )
}