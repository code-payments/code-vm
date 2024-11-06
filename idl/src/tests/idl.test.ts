import * as anchor from "@coral-xyz/anchor";
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import { PublicKey, Keypair, Connection, LAMPORTS_PER_SOL } from '@solana/web3.js'
import { BorshAccountsCoder, BorshInstructionCoder } from '@coral-xyz/anchor';

function getProvider(url: string) {
    return new anchor.AnchorProvider(new Connection(url), new NodeWallet(Keypair.generate()), {});
}

(async () => {
    const url = "https://api.mainnet-beta.solana.com";
    const idl = await anchor.Program.fetchIdl("vmT2hAx4N2U6DyjYxgQHER4VGC8tHJCfHNsSepBKCJZ", getProvider(url));
    const coder = new BorshAccountsCoder(idl);

    const rawData = Buffer.from("AQAAAAAAAAAJvOLP61dnuXMZIkAy+81lbTmOf7FbEDMsXuYDCPoTogszOKCrLMhB1bAUvGo891YpGHSzGclRfZu/qeTpZh75hQIAAAAAAAAP+8snOJC5FsSQcmyaqW/e0C4gsXr3BpHhsvcUK6D0FwKJfiL1bOUw4lPpAYEEY3+fL28rESJCioo00uuAKMKQ/hX/AAAAAAA=", "base64")
    const account = idl.accounts?.find((accountType: any) =>
        (rawData as Buffer).slice(0, 8).equals(coder.accountDiscriminator(accountType.name))
    );

    const accountDef = idl.types.find((type: any) => type.name === account.name);
    const decodedAccountData = coder.decode(account.name, rawData);

    console.log(decodedAccountData);

    const coderIx = new BorshInstructionCoder(idl);

    const ixData = Buffer.from("CAsDAAAAYwABAAIAAwAAAAABAUgAAADht/AGvkNEfKn4vlBVuW1/80lU7ylo9iTGncrG8jTszDwbvMXVSgf4YcwbpHwrt0MzDCXnL6rSsKB0uExS3UsPoIYBAAAAAAA=", 'base64')
    const decodedIx = coderIx.decode(ixData);

    console.log(ixData);
    console.log('what', decodedIx);
    
    //console.log(coder.accountDiscriminator("CodeVmAccount"));
})();