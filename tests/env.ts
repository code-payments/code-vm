import { TestMemory } from "./utils/memory";
import { TestStorage } from "./utils/storage";
import { TestVm } from "./utils/vm";
import { TestDurableNonce } from "./utils/nonce";
import { TestUser } from "./utils/user";
import { TestRelay, TestRelayProof } from "./utils/relay";

export interface CodeVM {
    vm?: TestVm,

    vm_memory?: TestMemory,

    store_accounts?: TestStorage,

    relays?: TestRelay[],

    // Virtual accounts
    accounts?: TestUser[],
    nonces?: TestDurableNonce[],
    relayProofs?: TestRelayProof[],
}
