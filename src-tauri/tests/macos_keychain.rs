#![cfg(target_os = "macos")]

use keyring::{Entry, Error};
use std::process::{self, Command};

const SERVICE: &str = "nidalee.test.keychain";
const CHILD_ACCOUNT: &str = "NIDALEE_KEYCHAIN_TEST_ACCOUNT";

#[test]
fn keychain_persists_across_processes() {
    // The child must open a fresh handle, as happens after restarting the app.
    if let Ok(account) = std::env::var(CHILD_ACCOUNT) {
        assert_eq!(
            Entry::new(SERVICE, &account).unwrap().get_password().unwrap(),
            "fixture"
        );
        return;
    }
    let account = format!("test-{}", process::id());
    let entry = Entry::new(SERVICE, &account).unwrap();
    entry.set_password("fixture").unwrap();
    let immediate = Entry::new(SERVICE, &account).unwrap().get_password();
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "keychain_persists_across_processes"])
        .env(CHILD_ACCOUNT, &account)
        .status();
    entry.delete_credential().unwrap();
    assert_eq!(immediate.unwrap(), "fixture");
    assert!(child.unwrap().success());
    assert!(matches!(
        Entry::new(SERVICE, &account).unwrap().get_password(),
        Err(Error::NoEntry)
    ));
}
