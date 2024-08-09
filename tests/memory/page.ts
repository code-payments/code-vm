import * as beet from '@metaplex-foundation/beet'

export class Page {
  constructor(
    readonly is_allocated: number,
    readonly data: number[],
    readonly next_page: number,
  ) { }

  static readonly DATA_LEN = 32; // Assuming MemoryLayout.Mixed here...

  static readonly LEN =
    1 +  // is_allocated
    Page.DATA_LEN + // data
    1;   // next_page

  static readonly struct = new beet.BeetArgsStruct<Page>(
    [
      ['is_allocated', beet.u8],
      ['data', beet.uniformFixedSizeArray(beet.u8, Page.DATA_LEN)],
      ['next_page', beet.u8],
    ],
    'Page'
  )
}

