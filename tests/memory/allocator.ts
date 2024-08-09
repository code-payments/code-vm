import * as beet from '@metaplex-foundation/beet'
import { Sector } from './sector';

const MAX_ACCOUNTS = 100; // TODO: set this to 65_535;
const NUM_SECTORS = 2; // TODO: set this to 255;

export class AccountIndex {
  constructor(
    readonly size: number,
    readonly page: number,
    readonly sector: number,
  ) { }

  static readonly LEN =
    2 + // size
    1 + // page
    1;  // sector

  static readonly struct = new beet.BeetArgsStruct<AccountIndex>(
    [
      ['size', beet.u16],
      ['page', beet.u8],
      ['sector', beet.u8],
    ],
    'AccountIndex'
  )
}

export class AccountBuffer {
  constructor(
    readonly accounts: AccountIndex[],
    readonly sectors: Sector[],
    
  ) { }

  static readonly NUM_ACCOUNTS = MAX_ACCOUNTS;
  static readonly NUM_SECTORS = NUM_SECTORS;

  static readonly LEN =         // ~4.6MB
    AccountBuffer.NUM_ACCOUNTS * AccountIndex.LEN +
    AccountBuffer.NUM_SECTORS * Sector.LEN;

  static readonly struct = new beet.FixableBeetArgsStruct<AccountBuffer>(
    [
      ['accounts', beet.uniformFixedSizeArray(AccountIndex.struct, AccountBuffer.NUM_ACCOUNTS, false)],
      ['sectors', beet.uniformFixedSizeArray(Sector.struct, AccountBuffer.NUM_SECTORS, false)]
    ],
    'AccountBuffer'
  )
}