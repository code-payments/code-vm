#[cfg(not(feature = "no-entrypoint"))]
use {solana_security_txt::security_txt};

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Code VM",
    project_url: "https://flipcash.com",
    contacts: "email:security@flipcash.com",
    policy: "https://github.com/code-payments/code-vm/blob/main/SECURITY.md",
    source_code: "https://github.com/code-payments/code-vm",
    auditors: "OtterSec"
}
