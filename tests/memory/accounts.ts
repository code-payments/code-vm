import * as anchor from "@coral-xyz/anchor";
import * as beet from '@metaplex-foundation/beet'
import * as beetSolana from '@metaplex-foundation/beet-solana'

export class VirtualAccount {
  readonly variant: number;
  readonly account: VirtualDurableNonce | VirtualTimelockAccount | VirtualRelayAccount;

  constructor(data: Buffer) {
    this.variant = data.readUInt8(0);
    if (this.variant == 0) {
      this.account = VirtualDurableNonce.struct.deserialize(data.slice(1, VirtualDurableNonce.LEN + 1))[0];
    } else if (this.variant == 1) {
      this.account = VirtualTimelockAccount.struct.deserialize(data.slice(1, VirtualTimelockAccount.LEN + 1))[0];
    } else if (this.variant == 2) {
      this.account = VirtualRelayAccount.struct.deserialize(data.slice(1, VirtualRelayAccount.LEN + 1))[0];
    } else {
      throw new Error("Unknown variant");
    }
  }

  static pack(va: VirtualAccount): Buffer {
    let data: Buffer;
    if (va.variant == 0) {
      data = VirtualDurableNonce.struct.serialize(va.account as VirtualDurableNonce)[0];
    } else if (va.variant == 1) {
      data = VirtualTimelockAccount.struct.serialize(va.account as VirtualTimelockAccount)[0];
    } else if (va.variant == 2) {
      data = VirtualRelayAccount.struct.serialize(va.account as VirtualRelayAccount)[0];
    } else {
      throw new Error("Unknown variant");
    }

    const variant = Buffer.alloc(1);
    variant.writeUInt8(va.variant);
    return Buffer.concat([variant, data]);
  }

  pack(): Buffer {
    return VirtualAccount.pack(this);
  }
}

export class VirtualDurableNonce {
  constructor(
    readonly address: anchor.web3.PublicKey,
    readonly nonce: anchor.web3.PublicKey,
  ) { }

  static readonly struct = new beet.BeetArgsStruct<VirtualDurableNonce>(
    [
      ['address', beetSolana.publicKey],
      ['nonce', beetSolana.publicKey],
    ],
    'VirtualDurableNonce'
  )

  static readonly LEN =
    32 + // address
    32;  // nonce
}

export class VirtualTimelockAccount {
  constructor(
    readonly owner: anchor.web3.PublicKey,
    readonly nonce: anchor.web3.PublicKey,

    readonly tokenBump: number,
    readonly unlockBump: number,
    readonly receiptBump: number,

    readonly balance: beet.bignum,
    readonly bump: number,
  ) { }

  static readonly struct = new beet.BeetArgsStruct<VirtualTimelockAccount>(
    [
      ['owner', beetSolana.publicKey],
      ['nonce', beetSolana.publicKey],

      ['tokenBump', beet.u8],
      ['unlockBump', beet.u8],
      ['receiptBump', beet.u8],

      ['balance', beet.u64],
      ['bump', beet.u8],
    ],
    'VirtualTimelockAccount'
  )

  static readonly LEN =
    32 + // owner
    32 + // nonce
    1 +  // token_bump
    1 +  // unlock_bump
    1 +  // receipt_bump
    8 +  // balance
    1;   // bump
}

export class VirtualRelayAccount {
  constructor(
    readonly address: anchor.web3.PublicKey,
    readonly commitment: anchor.web3.PublicKey,
    readonly recentRoot: anchor.web3.PublicKey,
    readonly destination: anchor.web3.PublicKey,
  ) { }

  static readonly struct = new beet.BeetArgsStruct<VirtualRelayAccount>(
    [
      ['address', beetSolana.publicKey],
      ['commitment', beetSolana.publicKey],
      ['recentRoot', beetSolana.publicKey],
      ['destination', beetSolana.publicKey],
    ],
    'VirtualRelayAccount'
  )

  static readonly LEN =
    32 + // address
    32 + // commitment
    32 + // recentRoot
    32;  // destination
}
