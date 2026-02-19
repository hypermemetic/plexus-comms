use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, sqlite::SqliteConnectOptions, ConnectOptions, Row};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct EmailStorageConfig {
    pub db_path: PathBuf,
}

impl Default for EmailStorageConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("email_accounts.db"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub name: String,
    pub smtp: Option<SmtpAccountConfig>,
    pub imap: Option<ImapAccountConfig>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SmtpAccountConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ImapAccountConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub struct EmailStorage {
    pool: SqlitePool,
}

impl EmailStorage {
    pub async fn new(config: EmailStorageConfig) -> Result<Self, String> {
        let db_url = format!("sqlite:{}?mode=rwc", config.db_path.display());
        let connect_options: SqliteConnectOptions = db_url
            .parse()
            .map_err(|e| format!("Failed to parse database URL: {}", e))?;
        let connect_options = connect_options.disable_statement_logging();

        let pool = SqlitePool::connect_with(connect_options)
            .await
            .map_err(|e| format!("Failed to connect to database: {}", e))?;

        let storage = Self { pool };
        storage.run_migrations().await?;

        Ok(storage)
    }

    async fn run_migrations(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS email_accounts (
                name TEXT PRIMARY KEY,
                smtp_config TEXT,
                imap_config TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to run migrations: {}", e))?;

        Ok(())
    }

    pub async fn register_account(&self, account: EmailAccount) -> Result<(), String> {
        let smtp_json = account
            .smtp
            .as_ref()
            .map(|s| serde_json::to_string(s).unwrap());
        let imap_json = account
            .imap
            .as_ref()
            .map(|i| serde_json::to_string(i).unwrap());

        sqlx::query(
            r#"
            INSERT INTO email_accounts (name, smtp_config, imap_config, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET
                smtp_config = excluded.smtp_config,
                imap_config = excluded.imap_config,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&account.name)
        .bind(smtp_json)
        .bind(imap_json)
        .bind(account.created_at)
        .bind(account.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to register account: {}", e))?;

        Ok(())
    }

    pub async fn get_account(&self, name: &str) -> Result<Option<EmailAccount>, String> {
        let row = sqlx::query(
            r#"
            SELECT name, smtp_config, imap_config, created_at, updated_at
            FROM email_accounts
            WHERE name = ?
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get account: {}", e))?;

        match row {
            Some(row) => {
                let smtp_json: Option<String> = row.get("smtp_config");
                let imap_json: Option<String> = row.get("imap_config");

                let smtp = smtp_json
                    .and_then(|s| serde_json::from_str(&s).ok());
                let imap = imap_json
                    .and_then(|i| serde_json::from_str(&i).ok());

                Ok(Some(EmailAccount {
                    name: row.get("name"),
                    smtp,
                    imap,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_accounts(&self) -> Result<Vec<EmailAccount>, String> {
        let rows = sqlx::query(
            r#"
            SELECT name, smtp_config, imap_config, created_at, updated_at
            FROM email_accounts
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list accounts: {}", e))?;

        let mut accounts = Vec::new();
        for row in rows {
            let smtp_json: Option<String> = row.get("smtp_config");
            let imap_json: Option<String> = row.get("imap_config");

            let smtp = smtp_json.and_then(|s| serde_json::from_str(&s).ok());
            let imap = imap_json.and_then(|i| serde_json::from_str(&i).ok());

            accounts.push(EmailAccount {
                name: row.get("name"),
                smtp,
                imap,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(accounts)
    }

    pub async fn remove_account(&self, name: &str) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM email_accounts WHERE name = ?
            "#,
        )
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to remove account: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
