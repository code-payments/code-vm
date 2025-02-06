#[cfg(not(feature = "no-entrypoint"))]
use {solana_security_txt::security_txt};

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "code-vm",
    project_url: "https://getcode.com",
    contacts: "email:security@getcode.com,email:contact@getcode.com",
    policy: "https://github.com/code-payments/code-vm/blob/main/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/code-payments/code-vm",
    auditors: "OtterSec"
}
