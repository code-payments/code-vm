#pragma endian little

#define MAX_NAME_LEN 32

#define COMPRESSED_STATE_DEPTH 20
#define RELAY_STATE_DEPTH 63
#define RELAY_HISTORY_ITEMS 32

enum AccountType : u8 {
    CodeVmAccount = 1,
    MemoryAccount = 2,
    StorageAccount = 3,
    RelayAccount = 4,
    UnlockStateAccount = 5,
    WithdrawReceiptAccount = 6,
};

enum ItemState : u8 {
    Free = 0,
    Used = 1,
};

enum AccountDataType : u8 {
    Nonce = 0,
    Timelock = 1,
    Relay = 2,
};

// Type Definitions

struct Pubkey {
    u8 value[32];
};

struct Hash {
    u8 value[32];
};

struct Signature {
    u8 value[64];
};

struct TokenPool {
    Pubkey vault;
    u8 vault_bump;
};

struct SliceAllocator<T, auto N> {
    ItemState state[N];
    T data[N];
};

struct VirtualTimelockAccount {
    Pubkey owner;
    Hash instance;
    u8 token_bump;
    u8 unlock_bump;
    u8 withdraw_bump;
    u64 balance;
    u8 bump;
};

struct VirtualDurableNonce {
    Pubkey address;
    Hash value;
};

struct VirtualRelayAccount {
    Pubkey target;
    Pubkey destination;
};

struct VirtualAccount {
    AccountDataType type;

    if (type == AccountDataType::Nonce) {
        VirtualDurableNonce data;
    }
    else if (type == AccountDataType::Timelock) {
        VirtualTimelockAccount data;
    }
    else if (type == AccountDataType::Relay) {
        VirtualRelayAccount data;
    }
};

struct AccountDataHeader {
    u8 version;
    u16 account_size;
    u32 num_accounts;
};

struct AccountData {
    AccountDataHeader header;
    SliceAllocator<VirtualAccount, header.num_accounts> allocator;
};

struct CodeVmAccount {
    Pubkey authority;
    Pubkey mint;
    u64 slot;
    Hash poh;
    TokenPool omnibus;
    u8 lock_duration;  // in days
    u8 bump;
    u8 _padding[5]; // To align to 8 bytes
};

struct MemoryAccount {
    Pubkey vm;
    u8 name[MAX_NAME_LEN];
    u8 bump;

    AccountData data;
};

struct RelayHistory {
    Hash items[RELAY_HISTORY_ITEMS];
    u8 offset;
    u8 num_items;
    u8 _padding[6];
};

struct RelayTree {
    Hash root;
    Hash filled_subtrees[RELAY_STATE_DEPTH];
    Hash zero_values[RELAY_STATE_DEPTH];
    u64 next_index;
};

struct RelayAccount {
    Pubkey vm;
    u8 name[MAX_NAME_LEN];
    TokenPool treasury;
    u8 bump;
    u8 num_levels;
    u8 num_history;
    u8 _padding[4];
    RelayHistory recent_roots;
    RelayTree history;
};

struct CompressedState {
    Hash root;
    Hash filled_subtrees[COMPRESSED_STATE_DEPTH];
    Hash zero_values[COMPRESSED_STATE_DEPTH];
    u64 next_index;
};

struct StorageAccount {
    Pubkey vm;
    u8 name[MAX_NAME_LEN];
    u8 bump;
    u8 depth;
    u8 _padding[6];
    CompressedState state;
};

struct UnlockStateAccount {
    Pubkey vm;
    Pubkey owner;
    Pubkey address;
    s64 unlock_at;
    u8 bump;
    u8 state;
    u8 _padding[6];
};

struct WithdrawReceiptAccount {
    Pubkey unlock_pda;
    Hash nonce;
    u64 amount;
    u8 bump;
    u8 _padding[7];
};

struct Account {
    AccountType _type;
    u8 _padding[7];

    if (_type == AccountType::CodeVmAccount) {
        CodeVmAccount data;
    }
    else if (_type == AccountType::MemoryAccount) {
        MemoryAccount data;
    }
    else if (_type == AccountType::StorageAccount) {
        StorageAccount data;
    }
    else if (_type == AccountType::RelayAccount) {
        RelayAccount data;
    }
    else if (_type == AccountType::UnlockStateAccount) {
        UnlockStateAccount data;
    }
    else if (_type == AccountType::WithdrawReceiptAccount) {
        WithdrawReceiptAccount data;
    }
    else {
        // Unknown type, read the rest as raw bytes
        u8 data[];
    }
};

// Entry Point
Account account @ 0x00;