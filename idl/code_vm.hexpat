#pragma endian little

#define MAX_NAME_LEN 32
#define NUM_ACCOUNTS 1000

#define COMPRESSED_STATE_DEPTH 20
#define RELAY_STATE_DEPTH 63
#define RELAY_HISTORY_ITEMS 32

// Enums
enum AccountType : u8 {
    CodeVmAccount = 1,
    MemoryAccount = 2,
    StorageAccount = 3,
    RelayAccount = 4,
    UnlockStateAccount = 5,
    WithdrawReceiptAccount = 6,
};

enum VirtualAccount : u8 {
    Nonce = 0,
    Timelock = 1,
    Relay = 2,
};

enum ItemState : u8 {
    Empty = 0,
    Allocated = 1,
};

enum AccountDataType : u8 {
    Unknown = 0,
    Timelock = 1,
    Nonce = 2,
    Relay = 3,
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

// SimpleAllocator Template
struct SimpleAllocator<T, auto N> {
    ItemState state[N];
    T data[N];
};

// Virtual Accounts
struct VirtualTimelockAccount {
    VirtualAccount _type;
    Pubkey owner;
    Hash instance;
    u8 token_bump;
    u8 unlock_bump;
    u8 withdraw_bump;
    u64 balance;
    u8 bump;
};

struct VirtualDurableNonce {
    VirtualAccount _type;
    Pubkey address;
    Hash value;
};

struct VirtualRelayAccount {
    VirtualAccount _type;
    Pubkey target;
    Pubkey destination;
};

// AccountData
struct AccountData {
    AccountDataType type;

    if (type == AccountDataType::Timelock) {
        SimpleAllocator<VirtualTimelockAccount, NUM_ACCOUNTS> allocator;
    }
    else if (type == AccountDataType::Nonce) {
        SimpleAllocator<VirtualDurableNonce, NUM_ACCOUNTS> allocator;
    }
    else if (type == AccountDataType::Relay) {
        SimpleAllocator<VirtualRelayAccount, NUM_ACCOUNTS> allocator;
    }
};

// CodeVmAccount
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

// MemoryAccount
struct MemoryAccount {
    Pubkey vm;
    u8 name[MAX_NAME_LEN];
    u8 bump;
    u8 _padding[6];
    AccountData data;
};

// RelayHistory and RelayTree Types
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

// RelayAccount
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
    

// StorageAccount
struct StorageAccount {
    Pubkey vm;
    u8 name[MAX_NAME_LEN];
    u8 bump;
    u8 depth;
    u8 _padding[6];
    CompressedState state;
};

// UnlockStateAccount
struct UnlockStateAccount {
    Pubkey vm;
    Pubkey owner;
    Pubkey address;
    s64 unlock_at;
    u8 bump;
    u8 state;
    u8 _padding[6];
};

// WithdrawReceiptAccount
struct WithdrawReceiptAccount {
    Pubkey unlock_pda;
    Hash nonce;
    u64 amount;
    u8 bump;
    u8 _padding[7];
};

// Account
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