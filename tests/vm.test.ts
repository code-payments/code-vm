import { describe, it } from "vitest";
import { CodeVM } from "./env";
import { MemoryLayout } from "./instructions/vm";
import * as test from "./instructions";

let env: CodeVM;

describe("vm-init", async () => {
    it("should create a vm account", async () => {
        env = await test.vm.test_create_vm();
    });

    it("should create memory accounts", async () => {
        // We can tell the VM that we prefer a certain memory layout. For
        // example, a relay account takes up 129 bytes, while a timelock account
        // takes up 77 bytes. The maximum amount of indexable pages in a memory
        // account is 2^16. A memory account that is optimized for timelock
        // account won't be able to store the full 65k relay accounts as it will
        // need to split the account into multiple pages.

        await test.vm.test_create_memory(env, "temp_01", MemoryLayout.Timelock); // 77 byte layout
        await test.vm.test_create_memory(env, "temp_02", MemoryLayout.Nonce);    // 65 byte layout
        await test.vm.test_create_memory(env, "temp_03", MemoryLayout.Relay);    // 129 byte layout

        // Lets use this one for the rest of the tests (data size of 32 bytes per page)
        env = await test.vm.test_create_memory(env, "vm_memory_01", MemoryLayout.Mixed);
    });

    it("should resize memory accounts", async () => {
        env = await test.vm.test_resize_memory(env);
    });

    it("should create storage accounts", async () => {
        env = await test.vm.test_create_storage(env);
    });
});

describe("vm-system", async () => {
    it("should create virtual nonces", async () => {
        const account_offset = 10; // Where to start the account index
        env = await test.system.test_create_nonces(env, 10, account_offset);
    });

    it("should create virtual accounts", async () => {
        const account_offset = 20; // Where to start the account index
        env = await test.timelock.test_create_accounts(env, 10, account_offset);
    });

    it("should create relays", async () => {
        env = await test.relay.test_create_relays(env, 3, 10_000);
    });

    it("should create a lookup table", async () => {
        env = await test.system.test_create_lookup_table(env);
    });
});

describe("vm-timelock", async () => {
    it("should create unlock state", async () => {
        env = await test.timelock.test_create_unlock_state(env, 20);
    });

    it("should deposit from the user's ATA", async () => {
        env = await test.timelock.test_deposit_from_ata(env, 20, 50);
    });

    it("should deposit from the user's deposit address", async () => {
        env = await test.timelock.test_deposit_from_pda(env, 20, 50);
    });
});

describe("vm-exec", async () => {
    it("should transfer", async () => {
        env = await test.timelock.test_transfer(env, 
            20, // from_index (account 20)
            21, // to_index (account 21)
            10, // durable nonce (account 10)
            50, // amount
        );
        env = await test.timelock.test_transfer(env, 
            21, // from_index (account 21)
            22, // to_index (account 22)
            10, // durable nonce (account 10)
            50, // amount
        );
    });

    it("should transfer external", async () => {
        env = await test.timelock.test_transfer_external(env, 
            20, // from_index (account 20)
            21, // to_index (account 21) (external ATA account)
            10, // durable nonce (account 10)
            25, // amount
        );
    });

    it("should transfer multiple", async () => {
        env = await test.timelock.test_transfer_multiple(env, 
            [
                { from_index: 20, to_index: 21, vdn_index: 10, amount: 20, },
                { from_index: 21, to_index: 22, vdn_index: 11, amount: 19, },
                { from_index: 22, to_index: 23, vdn_index: 12, amount: 18, },
                { from_index: 23, to_index: 24, vdn_index: 13, amount: 17, },
                { from_index: 24, to_index: 25, vdn_index: 14, amount: 16, },
                { from_index: 25, to_index: 26, vdn_index: 15, amount: 15, },
                { from_index: 26, to_index: 27, vdn_index: 16, amount: 14, },
                { from_index: 27, to_index: 28, vdn_index: 17, amount: 13, },
            ]
        );
    });
});

describe("vm-compression", async () => {
    it("should compress nonce accounts", async () => {
        env = await test.system.test_compress_nonce_account(env, 10);
        env = await test.system.test_compress_nonce_account(env, 11);
    });

    it("should compress timelock accounts", async () => {
        env = await test.system.test_compress_timelock_account(env, 27);
        env = await test.system.test_compress_timelock_account(env, 28);
    });

    it("should decompress timelock accounts", async () => {
        env = await test.system.test_decompress_timelock_account(env, 27, 42);
        env = await test.system.test_decompress_timelock_account(env, 28, 40);
    });

    it("should decompress nonce accounts", async () => {
        env = await test.system.test_decompress_nonce_account(env, 10, 52);
        env = await test.system.test_decompress_nonce_account(env, 11, 50);
    });
});

describe("vm-relay", async () => {
    it("should do blind transfers (to virtual accounts)", async () => {
        env = await test.relay.test_blind_transfer_to_virtual_account(env, 
            0,   // relay/treasury
            42,  // virtual timelock account index
            10,  // amount
            70,  // virtual relay account index (proof of payment)
        );
        env = await test.relay.test_blind_transfer_to_virtual_account(env, 1, 40, 20, 71);
        env = await test.relay.test_blind_transfer_to_virtual_account(env, 2, 22, 30, 72);
    });

    it("should save root", async () => {
        env = await test.relay.test_save_relay_root(env, 0);
    });

    it("should do blind transfers (to real ata accounts)", async () => {
        env = await test.relay.test_blind_transfer(env, 0, 42, 30, 73);
        env = await test.relay.test_blind_transfer(env, 0, 40, 20, 74);
        env = await test.relay.test_blind_transfer(env, 0, 22, 10, 75);
    });

    it("should save root (again)", async () => {
        env = await test.relay.test_save_relay_root(env, 0);
    });

    it("should compress/decompress a relay proof account", async () => {
        env = await test.system.test_compress_relay_account(env, 70);
        env = await test.system.test_decompress_relay_account(env, 70, 76);
    });
});