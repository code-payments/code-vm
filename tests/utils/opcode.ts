import * as anchor from "@coral-xyz/anchor";
import * as beet from '@metaplex-foundation/beet'
import * as beetSolana from '@metaplex-foundation/beet-solana'

export enum OpCode {
  System_Unknown = 0,

  Timelock_TransferToExternal = 10,
  Timelock_TransferToInternal = 11,
  Timelock_TransferToRelay = 12,
  Timelock_WithdrawToExternal = 13,
  Timelock_WithdrawToInternal = 14,

  Splitter_TransferToExternal = 20,
  Splitter_TransferToInternal = 21,
}

type OpCodeData =
  | { opcode: OpCode.Timelock_TransferToExternal; data: Timelock_TransferData }
  | { opcode: OpCode.Timelock_TransferToInternal; data: Timelock_TransferData }
  | { opcode: OpCode.Timelock_TransferToRelay; data: Timelock_TransferData }
  | { opcode: OpCode.Timelock_WithdrawToExternal; data: Timelock_WithdrawData }
  | { opcode: OpCode.Timelock_WithdrawToInternal; data: Timelock_WithdrawData }
  | { opcode: OpCode.Splitter_TransferToExternal; data: Relay_TransferData }
  | { opcode: OpCode.Splitter_TransferToInternal; data: Relay_TransferData };

export function serialize({ opcode, data }: OpCodeData) {
  switch (opcode) {
    case OpCode.Timelock_TransferToExternal:
      return Timelock_TransferData.struct.serialize(data);
    case OpCode.Timelock_TransferToInternal:
      return Timelock_TransferData.struct.serialize(data);
    case OpCode.Timelock_TransferToRelay:
      return Timelock_TransferData.struct.serialize(data);
    case OpCode.Timelock_WithdrawToExternal:
      return Timelock_WithdrawData.struct.serialize(data);
    case OpCode.Timelock_WithdrawToInternal:
      return Timelock_WithdrawData.struct.serialize(data);
    case OpCode.Splitter_TransferToExternal:
      return Relay_TransferData.struct.serialize(data);
    case OpCode.Splitter_TransferToInternal:
      return Relay_TransferData.struct.serialize(data);

    default:
      throw new Error(`Unknown opcode: ${opcode}`);
  }
}


class Timelock_TransferData {
  constructor(
    readonly signature: number[],
    readonly amount: beet.bignum,
  ) { }

  static readonly struct = new beet.BeetArgsStruct<Timelock_TransferData>(
    [
      ['signature', beet.uniformFixedSizeArray(beet.u8, 64)],
      ['amount', beet.u32],
    ],
    'Timelock_TransferData'
  )
}

class Timelock_WithdrawData {
  constructor(
    readonly signature: number[],
  ) { }

  static readonly struct = new beet.BeetArgsStruct<Timelock_WithdrawData>(
    [
      ['signature', beet.uniformFixedSizeArray(beet.u8, 64)],
    ],
    'Timelock_WithdrawData'
  )
}

class Relay_TransferData {
  constructor(
    readonly amount: beet.bignum,
    readonly transcript: anchor.web3.PublicKey,
    readonly recent_root: anchor.web3.PublicKey,
    readonly commitment: anchor.web3.PublicKey,
  ) { }

  static readonly struct = new beet.BeetArgsStruct<Relay_TransferData>(
    [
      ['amount', beet.u32],
      ['transcript', beetSolana.publicKey],
      ['recent_root', beetSolana.publicKey],
      ['commitment', beetSolana.publicKey],
    ],
    'Relay_TransferWithCommitmentData'
  )
}