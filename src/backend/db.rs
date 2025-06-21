#![allow(dead_code)]
use std::{
    mem::MaybeUninit,
    path::PathBuf,
    fs,
};
use sqlx::SqlitePool;
use chrono::{
    DateTime, 
    NaiveDateTime, 
    Utc
};
use tokio_xmpp::jid::{self, BareJid};
use crate::common::ContactId;
use crate::common::{AccountId, Password};


pub static mut MY_POOL: MaybeUninit<SqlitePool> = MaybeUninit::uninit();


pub async fn setup() -> anyhow::Result<()> {
    let path = get_path();
    let path = path.to_string_lossy();
    let path = format!("sqlite:{}", path);

    log::info!("db path: {}", path);

    let pool = SqlitePool::connect(path.as_str()).await?;

    unsafe {
        MY_POOL = MaybeUninit::new(pool.clone());
    }

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    Ok(())
}

#[inline]
#[allow(static_mut_refs)]
pub fn pool_ref() -> &'static SqlitePool {
    unsafe { MY_POOL.assume_init_ref() }
}

#[cfg(target_os="linux")]
/// creates db if not exit. panics on error.
pub fn get_path() -> PathBuf {

    let home = match std::env::var("HOME") {
        Ok(v) => v,
        Err(_e) => panic!("Couldn't find $HOME var. Cannot run as root."),
    };

    let mut path = PathBuf::from(home);

    path.push(".local/share/akama");

    let _ = std::fs::create_dir(path.as_path());
    path.push("db.db");
    let _ = fs::File::create_new(path.as_path());

    path
}

/// For now, create db in the current directy
#[cfg(not(target_os="linux"))]
pub fn get_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("db.db");
    let _ = fs::File::create_new(path.as_path());

    path
}

pub async fn get_accounts() -> Vec<(AccountId, Password)> {
    sqlx::query!(
        r#"
SELECT jid, password FROM account;
    "#
    )
    .fetch_all(pool_ref())
    .await
    .unwrap()
    .into_iter()
    .map(|r| (BareJid::new(&r.jid.unwrap()).unwrap().into(), r.password))
    .collect()
}

pub async fn add_account(
    jid: &AccountId, 
    _resource: Option<&jid::ResourceRef>, 
    password: &str
) {
    let jid = jid.as_str();

    sqlx::query!(
        r#"
INSERT INTO account ( jid, password )
VALUES (?1, ?2);
    "#,
        jid,
        password
    )
    .execute(pool_ref())
    .await
    .unwrap();
}

pub async fn add_contact(account: &AccountId, contact: &ContactId) {
    let account_jid = account.to_string();
    let contact_jid = contact.to_string();

    sqlx::query!(
        r#"
INSERT INTO contact ( account_jid, contact_jid )
VALUES (?1, ?2);
    "#,
        account_jid,
        contact_jid
    )
    .execute(pool_ref())
    .await
    .unwrap();
}

pub async fn get_contacts(account: &AccountId) -> Vec<ContactId> {
    let account = account.to_string();
    sqlx::query!(
        r#"
SELECT contact_jid FROM contact WHERE account_jid = ?1;
    "#,
        account
    )
    .fetch_all(pool_ref())
    .await
    .unwrap()
    .into_iter()
    .map(|r| BareJid::new(&r.contact_jid).unwrap().into())
    .collect()
}

pub async fn get_messages(
    account: &AccountId,
    contact: &ContactId,
) -> Vec<(bool, String, NaiveDateTime)> {
    let account = account.to_string();
    let contact = contact.to_string();
    sqlx::query!(
        r#"
SELECT "from", content, timestamp
FROM message
WHERE ("from" = ?1 AND "to" = ?2)
   OR ("from" = ?2 AND "to" = ?1)
ORDER BY timestamp ASC;
    "#,
        account,
        contact
    )
    .fetch_all(pool_ref())
    .await
    .unwrap()
    .into_iter()
    .map(|r| (account == r.from, r.content, r.timestamp))
    .collect()
}

pub async fn add_text_message(from: &BareJid, to: &BareJid, body: &str, timestamp: &DateTime<Utc>) {
    let pool = pool_ref();

    let by = from.to_string();
    let to = to.to_string();

    sqlx::query!(
        r#"
INSERT INTO message ("from", "to", content, timestamp)
VALUES (?1, ?2, ?3, ?4); 
    "#,
        by,
        to,
        body,
        timestamp
    )
    .execute(pool)
    .await
    .unwrap();
}
