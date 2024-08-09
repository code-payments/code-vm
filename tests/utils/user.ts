import * as token from "@solana/spl-token";
import * as anchor from "@coral-xyz/anchor";
import * as programs from "@code-wallet/programs";
import * as ed25519 from '@noble/ed25519';
import { sha512 } from '@noble/hashes/sha512';
import { sha256 } from '@noble/hashes/sha256';
import { TestVm } from "./vm";
import { OpCode, serialize } from "./opcode";
import { vm_timelock_exec, vm_timelock_exec_multiple } from "./exec";
import { TestMemory } from "./memory";
import { TestDurableNonce } from "./nonce";
import { compileMessage } from "./transaction";

ed25519.etc.sha512Sync = (...m) => sha512(ed25519.etc.concatBytes(...m));

export type TestUser = {
    keypair: anchor.web3.Keypair;
    owner: anchor.web3.PublicKey;
    ata: anchor.web3.PublicKey;

    virtual_timelock_address: anchor.web3.PublicKey;
    virtual_timelock_bump: number;

    virtual_vault_address: anchor.web3.PublicKey;
    virtual_vault_bump: number;

    unlock_pda: anchor.web3.PublicKey;
    unlock_bump: number;

    memory_index: number;

    data_hash?: Buffer;
};


function getAddressesFor(env: TestVm, owner: anchor.web3.PublicKey) {
    const { program, signer, vm, mint, lock_duration} = env;

    const vm_authority = signer;
    const timelockId = new anchor.web3.PublicKey("time2Z2SCnn3qYg3ULKVtdkh8YmZ5jFdKicnA1W2YnJ")

    const [
        virtual_timelock_address, 
        virtual_timelock_bump
    ] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("timelock_state"),
            mint.toBuffer(),
            vm_authority.publicKey.toBuffer(),
            owner.toBuffer(),
            Buffer.from([lock_duration])
        ],
        timelockId
    )

    const version_1 = 3;
    const [
        virtual_vault_address, 
        virtual_vault_bump
    ] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("timelock_vault"),
            virtual_timelock_address.toBuffer(),
            Buffer.from([version_1])
        ],
        timelockId
    )

    const [
        unlock_pda, 
        unlock_bump
    ] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            Buffer.from("vm_unlock_pda_account"),

            owner.toBuffer(),
            virtual_timelock_address.toBuffer(),
            vm.toBuffer(),
        ],
        program.programId
    )

    return {
        virtual_timelock_address,
        virtual_timelock_bump,
        virtual_vault_address,
        virtual_vault_bump,
        unlock_pda,
        unlock_bump,
    };
}


async function createUser(env: TestVm, memory_index: number = -1) : Promise<TestUser> {
    const { connection, signer, mint } = env;

    const vm_authority = signer;
    const owner = anchor.web3.Keypair.generate();

    const {
        virtual_timelock_address,
        virtual_timelock_bump,
        virtual_vault_address,
        virtual_vault_bump,
        unlock_pda,
        unlock_bump,
    } = getAddressesFor(env, owner.publicKey);

    const ata = await token.createAccount(connection, vm_authority, mint, owner.publicKey);
    await token.mintTo(connection, signer, mint, ata, vm_authority, 10_000);

    return {
        keypair: owner,
        owner: owner.publicKey,
        ata,
        virtual_timelock_address,
        virtual_timelock_bump,
        virtual_vault_address,
        virtual_vault_bump,
        unlock_pda,
        unlock_bump,
        memory_index,
    };
}

async function createVirtualAccount(
    env: TestVm, mem: TestMemory, user: TestUser
) {
    const { program, connection, signer, vm } = env;
    const vm_authority = signer;

    const tx = await program.methods
        .systemTimelockInit(
            user.memory_index,
            user.virtual_timelock_bump,
            user.virtual_vault_bump,
            user.unlock_bump,
        )
        .accountsPartial({
            vm,
            vmAuthority: vm_authority.publicKey,
            vmMemory: mem.vm_memory_account,
            virtualAccountOwner: user.owner,
        })
        .signers([signer])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);
}

async function createUnlockPda(env: TestVm, user: TestUser) {
    const { program, connection, signer, vm } = env;
    const vm_authority = signer;

    const tx = await program.methods
        .timelockUnlockInit(
            user.virtual_timelock_bump,
        )
        .accountsPartial({
            vm,
            virtualAccountOwner: user.keypair.publicKey,
            virtualAccount: user.virtual_timelock_address,
            unlockPda: user.unlock_pda,
            payer: vm_authority.publicKey,
        })
        .signers([
            vm_authority,   // paying for this transaction
            user.keypair, // authority of the virtual account
        ])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);
}

async function unlockUnlockPda(env: TestVm, user: TestUser) {
    const { program, connection, signer, vm } = env;
    const vm_authority = signer;

    const tx = await program.methods
        .timelockUnlockRequest(
        )
        .accountsPartial({
            vm: vm,
            unlockPda: user.unlock_pda,
            virtualAccountOwner: user.keypair.publicKey,
            payer: vm_authority.publicKey,
        })
        .signers([
            vm_authority,    // paying for this transaction
            user.keypair,  // authority of the virtual account
        ])
        .rpc({ skipPreflight: false });

    await connection.confirmTransaction(tx);

    console.log("tx", tx);
}

async function depositFromAta(
    vm: TestVm,
    mem: TestMemory,
    vta: TestUser,
    amount: number
    ) {
    const { program, connection, signer } = vm;

    // This deposit flow requires both the user signature and the vm authority
    // signature. It assumes that tokens are already in the user's ATA.

    const tx = program.methods
        .timelockDepositFromAta(
            vta.memory_index,
            new anchor.BN(amount),
        )
        .accountsPartial({
            vm: vm.vm,
            vmAuthority: signer.publicKey,
            vmOmnibus: vm.omnibus,
            vmMemory: mem.vm_memory_account,
            depositor: vta.keypair.publicKey,
            depositorAta: vta.ata,
        })
        .signers([signer, vta.keypair]);

    const sig = await tx.rpc({ skipPreflight: true })

    await connection.confirmTransaction(sig);

    console.log("tx", sig);
}

async function depositFromPda(
    vm: TestVm,
    mem: TestMemory,
    vta: TestUser,
    amount: number
    ) {
    const { program, connection, signer, mint } = vm;

    // This deposit flow is a bit more complex than the previous one. It assumes
    // that we have given the user a special address that a standard wallet will
    // auto-create an ATA against.

    const [
        depositPda, 
        depositBump
    ] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("code-vm"),
            Buffer.from("vm_deposit_pda"),
            vta.owner.toBuffer(),
            vm.vm.toBuffer(),
        ],
        program.programId
    )

    /*
    // Can't do this as it hard-codes a allowOffCurve flag to false
    const depositAta = await createAssociatedTokenAccount(connection,
        signer,
        mint,
        depositPda,
        { skipPreflight: true, }
    );
    */

    const depositAta = token.getAssociatedTokenAddressSync(
        mint,
        depositPda,
        true,
    );

    let sig = await connection.sendTransaction(
        new anchor.web3.Transaction().add(
            token.createAssociatedTokenAccountInstruction(
                signer.publicKey,
                depositAta,
                depositPda,
                mint,
            )
        ), 
        [signer], 
        { skipPreflight: true }
    );
    await connection.confirmTransaction(sig);

    // Simulate deposit from an external wallet (phantom, solflare, etc...).
    // These wallets will create-on-transfer the ATA
    sig = await token.transfer(
        connection, 
        signer, 
        vta.ata,
        depositAta,
        vta.keypair,
        amount,
    );
    await connection.confirmTransaction(sig);

    // Deposit from the PDA (ATA) into the VTA
    const tx = program.methods
        .timelockDepositFromPda(
            vta.memory_index,
            new anchor.BN(amount),
            depositBump,
        )
        .accountsPartial({
            vm: vm.vm,
            vmAuthority: signer.publicKey,
            vmOmnibus: vm.omnibus,
            vmMemory: mem.vm_memory_account,
            depositor: vta.owner,
            depositPda,
            depositAta,
        })
        .signers([ signer ]); // The depositor does not need to sign 

    sig = await tx.rpc({ skipPreflight: true })

    await connection.confirmTransaction(sig);

    console.log("tx", sig);
}

async function transferWithAuthority(
    vm: TestVm,
    memory: TestMemory,
    source_user: TestUser,
    dest_user: TestUser,
    vdn: TestDurableNonce,
    amount: number
) {
    const { opcode, data, mem_indicies, mem_banks } = 
        await createTransferInstruction(vm, source_user, dest_user, vdn, amount);

    await vm_timelock_exec(
        vm, 
        [memory, null, null, null],
        opcode, 
        data, 
        mem_indicies, 
        mem_banks,
        null,
    );
}

async function transferWithAuthorityMultiple(
    vm: TestVm,
    memory: TestMemory,
    actions: {
        source_user: TestUser,
        dest_user: TestUser,
        vdn: TestDurableNonce,
        amount: number
    }[]
) {
    const ix = await Promise.all(actions.map(async (action) => {
        const { opcode, data, mem_indicies, mem_banks } = 
            await createTransferInstruction(vm, action.source_user, action.dest_user, action.vdn, action.amount);

        return {
            opcode,
            data, 
            mem_indicies, 
            mem_banks,
            externalAddress: null,
        };
    }));

    await vm_timelock_exec_multiple(
        vm, 
        [memory, null, null, null],
        ix,
    );
}

async function transferWithAuthorityExternal(
    vm: TestVm,
    memory: TestMemory,
    source_user: TestUser,
    dest_user: TestUser,
    vdn: TestDurableNonce,
    amount: number
) {
    const destination = dest_user.ata;
    const { opcode, data, mem_indicies, mem_banks, externalAddress } = 
        await createTransferInstruction(vm, source_user, destination, vdn, amount, true);

    await vm_timelock_exec(
        vm, 
        [memory, null, null, null],
        opcode, 
        data, 
        mem_indicies, 
        mem_banks, 
        externalAddress, 
    );
}


async function createTransferInstruction(
    vm: TestVm,
    source_user: TestUser,
    dest_user: TestUser | anchor.web3.PublicKey, // Can be TestUser or PublicKey
    vdn: TestDurableNonce,
    amount: number,
    isExternal: boolean = false // Flag to handle external transfer case
) {
    const blockhash = await vdn.getValue();
    const destination = isExternal ? 
        dest_user as anchor.web3.PublicKey : 
        (dest_user as TestUser).virtual_vault_address;

    const tx = new anchor.web3.Transaction();
    tx.add(
        anchor.web3.SystemProgram.nonceAdvance({
            noncePubkey: vdn.address,
            authorizedPubkey: vm.signer.publicKey,
        }),
        new anchor.web3.TransactionInstruction({
            keys: [],
            data: Buffer.from("ZTAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=", "utf-8"),
            programId: new anchor.web3.PublicKey("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo"),
        }),
        programs.timelock.createTransferWithAuthorityInstruction(
            {
                timelock: source_user.virtual_timelock_address,
                vault: source_user.virtual_vault_address,
                vaultOwner: source_user.keypair.publicKey,
                timeAuthority: vm.signer.publicKey,
                payer: vm.signer.publicKey,
                destination,
            },
            {
                amount,
                timelockBump: source_user.virtual_timelock_bump,
            },
        ),
    );

    tx.recentBlockhash = blockhash.toBase58();
    tx.feePayer = vm.signer.publicKey;

    const msg = compileMessage(tx).serialize();
    const hash = sha256(msg);
    const signature = [...ed25519.sign(hash, source_user.keypair.secretKey.subarray(0, 32))];

    const opcode = isExternal ? 
        OpCode.Timelock_TransferToExternal : 
        OpCode.Timelock_TransferToInternal;

    const mem_indicies = isExternal ? 
        [vdn.index, source_user.memory_index] : 
        [vdn.index, source_user.memory_index, 
            (dest_user as TestUser).memory_index];

    const mem_banks = isExternal ? 
        [0, 0] : 
        [0, 0, 0];

    const [data] = serialize({ opcode, data: { signature, amount } });
    const externalAddress = isExternal ? destination : null;
    const unlockPda = source_user.unlock_pda;

    return { 
        opcode,
        data, 
        mem_indicies, 
        mem_banks, 
        unlockPda, 
        externalAddress, 
    };
}


export {
    getAddressesFor,
    createUser,
    createVirtualAccount,

    createUnlockPda,
    unlockUnlockPda,

    depositFromAta,
    depositFromPda,

    transferWithAuthority,
    transferWithAuthorityMultiple,

    transferWithAuthorityExternal,
}