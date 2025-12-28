use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::domain::{TransactionStatus, TransactionType, Wallet};
use crate::repository::{TransactionRepository, WalletRepository};
use crate::services::solana::SolanaClient;
use crate::services::webhook::WebhookService;

/// Interval between sync cycles
const SYNC_INTERVAL: Duration = Duration::from_secs(30);

/// Number of recent transactions to fetch per wallet
const SYNC_LIMIT: usize = 20;

pub struct SyncService {
    pool: PgPool,
    solana_client: Arc<SolanaClient>,
    webhook_service: Arc<WebhookService>,
    shutdown: Arc<AtomicBool>,
}

#[derive(Debug, Default)]
pub struct SyncReport {
    pub wallets_synced: u32,
    pub new_transactions: u32,
    pub webhooks_triggered: u32,
    pub errors: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl SyncService {
    pub fn new(
        pool: PgPool,
        solana_client: Arc<SolanaClient>,
        webhook_service: Arc<WebhookService>,
    ) -> Self {
        Self {
            pool,
            solana_client,
            webhook_service,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the background sync loop
    pub fn start_background_sync(self: Arc<Self>) -> JoinHandle<()> {
        let service = self.clone();

        tokio::spawn(async move {
            info!("Background sync service started");

            loop {
                // Check for shutdown signal
                if service.shutdown.load(Ordering::Relaxed) {
                    info!("Background sync service shutting down");
                    break;
                }

                // Run sync cycle
                match service.sync_all_wallets().await {
                    Ok(report) => {
                        if report.new_transactions > 0 || !report.errors.is_empty() {
                            info!(
                                wallets = report.wallets_synced,
                                new_txs = report.new_transactions,
                                webhooks = report.webhooks_triggered,
                                errors = report.errors.len(),
                                "Sync cycle completed"
                            );
                        }
                    }
                    Err(e) => {
                        error!("Sync cycle failed: {}", e);
                    }
                }

                // Also retry any pending webhooks
                match service.webhook_service.retry_pending_webhooks().await {
                    Ok(retried) if retried > 0 => {
                        info!(count = retried, "Retried pending webhooks");
                    }
                    Err(e) => {
                        error!("Failed to retry pending webhooks: {}", e);
                    }
                    _ => {}
                }

                // Wait for next cycle
                tokio::time::sleep(SYNC_INTERVAL).await;
            }
        })
    }

    /// Signal the background sync to stop
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    /// Sync all registered wallets
    pub async fn sync_all_wallets(&self) -> Result<SyncReport, crate::error::AppError> {
        let mut report = SyncReport {
            started_at: Some(Utc::now()),
            ..Default::default()
        };

        // Get all registered wallets
        let wallets = WalletRepository::list_all(&self.pool).await?;

        for wallet in wallets {
            match self.sync_wallet(&wallet).await {
                Ok((new_txs, webhooks)) => {
                    report.wallets_synced += 1;
                    report.new_transactions += new_txs;
                    report.webhooks_triggered += webhooks;
                }
                Err(e) => {
                    let error_msg = format!("Failed to sync wallet {}: {}", wallet.address, e);
                    warn!("{}", error_msg);
                    report.errors.push(error_msg);
                }
            }
        }

        report.completed_at = Some(Utc::now());
        Ok(report)
    }

    /// Sync a single wallet and return (new_transactions, webhooks_triggered)
    async fn sync_wallet(&self, wallet: &Wallet) -> Result<(u32, u32), crate::error::AppError> {
        let mut new_txs = 0u32;
        let mut webhooks = 0u32;

        // Fetch recent transactions from Solana
        let parsed_txs = self
            .solana_client
            .sync_wallet_transactions(&wallet.address, SYNC_LIMIT)
            .await?;

        for parsed in parsed_txs {
            // Check if we already have this transaction
            if TransactionRepository::exists(&self.pool, &parsed.signature).await? {
                continue;
            }

            // Determine transaction type
            let tx_type = match parsed.tx_type.as_str() {
                "send" => TransactionType::Send,
                "receive" => TransactionType::Receive,
                _ => continue,
            };

            // Store the transaction
            let transaction = TransactionRepository::create(
                &self.pool,
                &parsed.signature,
                &wallet.address,
                tx_type,
                parsed.amount,
                &self.solana_client.usdc_mint,
                &parsed.counterparty,
                TransactionStatus::Confirmed,
                parsed.block_time,
            )
            .await;

            // Handle the case where ON CONFLICT DO NOTHING returns no rows
            let transaction = match transaction {
                Ok(tx) => tx,
                Err(crate::error::AppError::Database(sqlx::Error::RowNotFound)) => {
                    // Transaction already exists (race condition), skip
                    continue;
                }
                Err(e) => return Err(e),
            };

            new_txs += 1;
            info!(
                wallet = %wallet.address,
                signature = %transaction.signature,
                tx_type = %transaction.tx_type,
                amount = %transaction.amount,
                "New transaction detected"
            );

            // Trigger webhook for receive transactions
            if matches!(tx_type, TransactionType::Receive) {
                if let Err(e) = self
                    .webhook_service
                    .notify_payment_received(wallet, &transaction)
                    .await
                {
                    warn!(
                        wallet = %wallet.address,
                        signature = %transaction.signature,
                        error = %e,
                        "Failed to send webhook notification"
                    );
                } else {
                    webhooks += 1;
                }
            }
        }

        Ok((new_txs, webhooks))
    }
}

impl serde::Serialize for SyncReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SyncReport", 6)?;
        state.serialize_field("wallets_synced", &self.wallets_synced)?;
        state.serialize_field("new_transactions", &self.new_transactions)?;
        state.serialize_field("webhooks_triggered", &self.webhooks_triggered)?;
        state.serialize_field("errors", &self.errors)?;
        state.serialize_field("started_at", &self.started_at)?;
        state.serialize_field("completed_at", &self.completed_at)?;
        state.end()
    }
}
