import * as beet from '@metaplex-foundation/beet'
import { Page } from './page';

export class Sector {
  constructor(
    readonly num_allocated: number,
    readonly pages: Page[],
  ) { }

  static readonly NUM_PAGES = 255;
  static readonly LEN =
    1 +                           // num_allocated
    Sector.NUM_PAGES * Page.LEN;  // pages

  static readonly struct = new beet.BeetArgsStruct<Sector>(
    [
      ['num_allocated', beet.u8],
      ['pages', beet.uniformFixedSizeArray(Page.struct, Sector.NUM_PAGES, false)],
    ],
    'Sector'
  )
}

