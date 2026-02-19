use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, sqlite::SqliteConnectOptions, ConnectOptions, Row};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DiscordStorageConfig {
    pub db_path: PathBuf,
}

impl Default for DiscordStorageConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("discord_accounts.db"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DiscordAccountConfig {
    pub bot_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordAccount {
    pub name: String,
    pub bot_token: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct DiscordStorage {
    pool: SqlitePool,
}

impl DiscordStorage {
    pub async fn new(config: DiscordStorageConfig) -> Result<Self, String> {
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
            CREATE TABLE IF NOT EXISTS discord_accounts (
                name TEXT PRIMARY KEY,
                bot_token TEXT NOT NULL,
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

    pub async fn register_account(&self, account: DiscordAccount) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO discord_accounts (name, bot_token, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET
                bot_token = excluded.bot_token,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&account.name)
        .bind(&account.bot_token)
        .bind(account.created_at)
        .bind(account.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to register account: {}", e))?;

        Ok(())
    }

    pub async fn get_account(&self, name: &str) -> Result<Option<DiscordAccount>, String> {
        let row = sqlx::query(
            r#"
            SELECT name, bot_token, created_at, updated_at
            FROM discord_accounts
            WHERE name = ?
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get account: {}", e))?;

        match row {
            Some(row) => Ok(Some(DiscordAccount {
                name: row.get("name"),
                bot_token: row.get("bot_token"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            None => Ok(None),
        }
    }

    pub async fn list_accounts(&self) -> Result<Vec<DiscordAccount>, String> {
        let rows = sqlx::query(
            r#"
            SELECT name, bot_token, created_at, updated_at
            FROM discord_accounts
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list accounts: {}", e))?;

        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(DiscordAccount {
                name: row.get("name"),
                bot_token: row.get("bot_token"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(accounts)
    }

    pub async fn remove_account(&self, name: &str) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM discord_accounts WHERE name = ?
            "#,
        )
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to remove account: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
