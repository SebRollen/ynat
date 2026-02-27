use crate::cache::Cache;
use crate::events::DataEvent;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use ynab_api::{
    endpoints::{
        transactions::{NewTransaction, TransactionUpdate},
        BudgetId, TransactionId,
    },
    Client, Request,
};

/// Data loader that implements cache-first loading with delta updates
#[derive(Clone)]
pub struct DataLoader {
    pub api_client: Arc<Client>,
    pub cache: Arc<Cache>,
    pub data_tx: mpsc::UnboundedSender<DataEvent>,
}

impl DataLoader {
    pub fn new(
        api_client: Arc<Client>,
        cache: Arc<Cache>,
        data_tx: mpsc::UnboundedSender<DataEvent>,
    ) -> Self {
        Self {
            api_client,
            cache,
            data_tx,
        }
    }

    /// Load budgets with cache-first strategy
    pub async fn load_budgets(&self, force_refresh: bool, include_accounts: bool) {
        tracing::info!("Loading budgets (force_refresh={})", force_refresh);

        // Step 1: Try cache first (fast path)
        if !force_refresh {
            if let Ok(Some(cached)) = self.cache.get_budgets().await {
                tracing::debug!("Loaded {} budgets from cache", cached.budgets.len());
                // Send cached data immediately
                let _ = self.data_tx.send(DataEvent::BudgetsCacheLoaded {
                    budgets: cached.budgets.clone(),
                    default_budget: cached.default_budget.clone(),
                });

                // TODO: Implement delta for budgets if YNAB API supports it
                // For now, we skip delta check for budgets
                return;
            } else {
                tracing::debug!("No cached budgets found");
            }
        }

        // Step 2: Load from API (slower path or forced refresh)
        tracing::debug!("Fetching budgets from API");
        let req = Request::budgets().list().include_accounts(include_accounts);
        match self.api_client.send(req).await {
            Ok(response) => {
                tracing::info!("Loaded {} budgets from API", response.data.budgets.len());
                // Send fresh data
                let _ = self.data_tx.send(DataEvent::BudgetsLoaded {
                    budgets: response.data.budgets.clone(),
                    default_budget: response.data.default_budget.clone(),
                });

                // Update cache in background
                let cache = self.cache.clone();
                let budgets = response.data.budgets;
                let default_budget = response.data.default_budget;
                tokio::spawn(async move {
                    let _ = cache.set_budgets(&budgets, default_budget).await;
                    tracing::debug!("Cached budgets updated");
                });
            }
            Err(e) => {
                tracing::error!("Failed to load budgets from API: {}", e);
                let _ = self.data_tx.send(DataEvent::LoadError {
                    error: e.to_string(),
                });
            }
        }
    }

    /// Load accounts with cache-first strategy and delta updates
    pub async fn load_accounts(&self, budget_id: String, force_refresh: bool) {
        tracing::info!(
            "Loading accounts for budget {} (force_refresh={})",
            budget_id,
            force_refresh
        );

        // Step 1: Try cache first (fast path)
        if !force_refresh {
            if let Ok(Some(cached)) = self.cache.get_accounts(&budget_id).await {
                tracing::debug!("Loaded {} accounts from cache", cached.accounts.len());
                // Send cached data immediately
                let _ = self.data_tx.send(DataEvent::AccountsCacheLoaded {
                    accounts: cached.accounts.clone(),
                });

                // Step 2: Check for delta updates in background
                if let Some(server_knowledge) = cached.server_knowledge {
                    tracing::debug!(
                        "Checking for account deltas (server_knowledge={})",
                        server_knowledge
                    );
                    self.check_accounts_delta(budget_id.clone(), server_knowledge.into())
                        .await;
                } else {
                    tracing::debug!("No server knowledge, fetching full accounts");
                    // No server_knowledge, need full refresh
                    self.fetch_accounts_full(budget_id.clone()).await;
                }
                return;
            } else {
                tracing::debug!("No cached accounts found");
            }
        }

        // Cache miss or forced refresh - load from API
        self.fetch_accounts_full(budget_id).await;
    }

    /// Check for delta updates to accounts
    async fn check_accounts_delta(&self, budget_id: String, last_knowledge: i64) {
        let budget_id_api: BudgetId = budget_id.clone().into();
        let req = Request::accounts()
            .with_budget(budget_id_api)
            .list()
            .last_knowledge_of_server(last_knowledge.into());
        match self.api_client.send(req).await {
            Ok(delta_response) => {
                // Check if there are actual changes
                if let Some(new_knowledge) = delta_response.data.server_knowledge {
                    if new_knowledge.inner() > last_knowledge {
                        tracing::info!(
                            "Found {} account changes (delta)",
                            delta_response.data.accounts.len()
                        );
                        // Send delta update
                        let _ = self.data_tx.send(DataEvent::AccountsDeltaLoaded {
                            delta: delta_response.data.accounts.clone(),
                        });

                        // Update cache in background
                        let cache = self.cache.clone();
                        let budget_id_clone = budget_id.clone();
                        let accounts = delta_response.data.accounts;
                        let new_knowledge_i64 = new_knowledge.inner();
                        tokio::spawn(async move {
                            let _ = cache
                                .merge_accounts_delta(
                                    &budget_id_clone,
                                    &accounts,
                                    new_knowledge_i64,
                                )
                                .await;
                        });
                    }
                }
            }
            Err(e) => {
                // Delta check failed, not critical (we have cached data)
                tracing::error!("Delta check failed for accounts: {}", e);
            }
        }
    }

    /// Fetch full accounts data from API
    async fn fetch_accounts_full(&self, budget_id: String) {
        tracing::debug!("Fetching full accounts from API");
        let budget_id_api: BudgetId = budget_id.clone().into();
        let req = Request::accounts().with_budget(budget_id_api).list();
        match self.api_client.send(req).await {
            Ok(response) => {
                tracing::info!("Loaded {} accounts from API", response.data.accounts.len());
                // Send fresh data
                let _ = self.data_tx.send(DataEvent::AccountsLoaded {
                    accounts: response.data.accounts.clone(),
                });

                // Update cache in background
                let cache = self.cache.clone();
                let budget_id_clone = budget_id.clone();
                let accounts = response.data.accounts;
                let server_knowledge = response.data.server_knowledge.map(|k| k.inner());
                tokio::spawn(async move {
                    let _ = cache
                        .set_accounts(&budget_id_clone, &accounts, server_knowledge)
                        .await;
                });
            }
            Err(e) => {
                let _ = self.data_tx.send(DataEvent::LoadError {
                    error: e.to_string(),
                });
            }
        }
    }

    /// Load transactions with cache-first strategy and delta updates
    pub async fn load_transactions(
        &self,
        budget_id: String,
        account_id: String,
        force_refresh: bool,
    ) {
        tracing::info!(
            "Loading transactions for budget {} account {} (force_refresh={})",
            budget_id,
            account_id,
            force_refresh
        );

        // Step 1: Try cache first (fast path)
        if !force_refresh {
            if let Ok(Some(cached)) = self.cache.get_transactions(&budget_id, &account_id).await {
                // Filter out deleted transactions
                let transactions: Vec<_> = cached
                    .transactions
                    .into_iter()
                    .filter(|t| !t.deleted)
                    .collect();

                tracing::debug!("Loaded {} transactions from cache", transactions.len());
                // Send cached data immediately
                let _ = self.data_tx.send(DataEvent::TransactionsCacheLoaded {
                    transactions: transactions.clone(),
                });

                // Step 2: Check for delta updates in background
                if let Some(server_knowledge) = cached.server_knowledge {
                    tracing::debug!(
                        "Checking for transaction deltas (server_knowledge={})",
                        server_knowledge
                    );
                    self.check_transactions_delta(
                        budget_id.clone(),
                        account_id.clone(),
                        server_knowledge,
                    )
                    .await;
                } else {
                    tracing::debug!("No server knowledge, fetching full transactions");
                    // No server_knowledge, need full refresh
                    self.fetch_transactions_full(budget_id.clone(), account_id.clone())
                        .await;
                }
                return;
            } else {
                tracing::debug!("No cached transactions found");
            }
        }

        // Cache miss or forced refresh - load from API
        self.fetch_transactions_full(budget_id, account_id).await;
    }

    /// Check for delta updates to transactions
    async fn check_transactions_delta(
        &self,
        budget_id: String,
        account_id: String,
        last_knowledge: i64,
    ) {
        let budget_id_api: BudgetId = budget_id.clone().into();
        let account_id_uuid = Uuid::parse_str(&account_id).expect("invalid account_id uuid");
        let req = Request::transactions()
            .with_budget(budget_id_api)
            .list(account_id_uuid)
            .last_knowledge_of_server(last_knowledge.into());
        match self.api_client.send(req).await {
            Ok(delta_response) => {
                // Check if there are actual changes
                if let Some(new_knowledge) = delta_response.data.server_knowledge {
                    if new_knowledge.inner() > last_knowledge {
                        // Filter out deleted transactions
                        let delta: Vec<_> = delta_response
                            .data
                            .transactions
                            .into_iter()
                            .filter(|t| !t.deleted)
                            .collect();

                        // Send delta update
                        let _ = self.data_tx.send(DataEvent::TransactionsDeltaLoaded {
                            delta: delta.clone(),
                        });

                        // Update cache in background
                        let cache = self.cache.clone();
                        let budget_id_clone = budget_id.clone();
                        let account_id_clone = account_id.clone();
                        let new_knowledge_i64 = new_knowledge.inner();
                        tokio::spawn(async move {
                            let _ = cache
                                .merge_transactions_delta(
                                    &budget_id_clone,
                                    &account_id_clone,
                                    &delta,
                                    new_knowledge_i64,
                                )
                                .await;
                        });
                    }
                }
            }
            Err(e) => {
                // Delta check failed, not critical (we have cached data)
                tracing::error!("Delta check failed for transactions: {}", e);
            }
        }
    }

    /// Fetch full transactions data from API
    async fn fetch_transactions_full(&self, budget_id: String, account_id: String) {
        let budget_id_api: BudgetId = budget_id.clone().into();
        let account_id_uuid = Uuid::parse_str(&account_id).expect("invalid account_id uuid");
        let req = Request::transactions()
            .with_budget(budget_id_api)
            .list(account_id_uuid);
        match self.api_client.send(req).await {
            Ok(response) => {
                // Filter out deleted transactions
                let transactions: Vec<_> = response
                    .data
                    .transactions
                    .into_iter()
                    .filter(|t| !t.deleted)
                    .collect();

                // Send fresh data
                let _ = self.data_tx.send(DataEvent::TransactionsLoaded {
                    transactions: transactions.clone(),
                });

                // Update cache in background
                let cache = self.cache.clone();
                let budget_id_clone = budget_id.clone();
                let account_id_clone = account_id.clone();
                let server_knowledge = response.data.server_knowledge.map(|k| k.inner());
                tokio::spawn(async move {
                    let _ = cache
                        .set_transactions(
                            &budget_id_clone,
                            &account_id_clone,
                            &transactions,
                            server_knowledge,
                        )
                        .await;
                });
            }
            Err(e) => {
                tracing::error!("Failed to load transactions from API: {}", e);
                let _ = self.data_tx.send(DataEvent::LoadError {
                    error: e.to_string(),
                });
            }
        }
    }

    /// Load plan with cache-first strategy
    pub async fn load_plan(&self, budget_id: String, force_refresh: bool) {
        tracing::info!(
            "Loading plan for budget {} (force_refresh={})",
            budget_id,
            force_refresh
        );

        // Step 1: Try cache first (fast path)
        if !force_refresh {
            if let Ok(Some(cached)) = self.cache.get_plan(&budget_id).await {
                tracing::debug!("Loaded {} categories from cache", cached.categories.len());
                // Send cached data immediately
                let _ = self.data_tx.send(DataEvent::PlanCacheLoaded {
                    month: cached.month.clone(),
                    categories: cached.categories.clone(),
                });

                // For now, we always fetch fresh data after showing cache
                // TODO: Implement delta updates if YNAB API supports server_knowledge for months
                tracing::debug!("Fetching fresh plan data from API");
                self.fetch_plan_full(budget_id).await;
                return;
            } else {
                tracing::debug!("No cached plan found");
            }
        }

        // Cache miss or forced refresh - load from API
        self.fetch_plan_full(budget_id).await;
    }

    /// Fetch full plan data from API
    async fn fetch_plan_full(&self, budget_id: String) {
        tracing::debug!("Fetching full plan from API");
        let budget_id_api: BudgetId = budget_id.clone().into();
        let req = Request::months().get().budget_id(budget_id_api);
        match self.api_client.send(req).await {
            Ok(response) => {
                tracing::info!(
                    "Loaded {} categories from API",
                    response.data.month.categories.len()
                );
                // Send fresh data
                let _ = self.data_tx.send(DataEvent::PlanLoaded {
                    month: response.data.month.clone(),
                    categories: response.data.month.categories.clone(),
                });

                // Update cache in background
                let cache = self.cache.clone();
                let budget_id_clone = budget_id.clone();
                let month = response.data.month;
                let server_knowledge = response.data.server_knowledge.map(|k| k.inner());
                tokio::spawn(async move {
                    let _ = cache
                        .set_plan(
                            &budget_id_clone,
                            &month,
                            &month.categories,
                            server_knowledge,
                        )
                        .await;
                    tracing::debug!("Cached plan updated");
                });
            }
            Err(e) => {
                tracing::error!("Failed to load plan from API: {}", e);
                let _ = self.data_tx.send(DataEvent::LoadError {
                    error: e.to_string(),
                });
            }
        }
    }

    /// Load plan for a specific month with cache-first strategy
    pub async fn load_plan_month(&self, budget_id: String, month: String) {
        tracing::info!("Loading plan for budget {} month {}", budget_id, month);

        // Try cache first
        if let Ok(Some(cached)) = self.cache.get_plan_month(&budget_id, &month).await {
            tracing::debug!(
                "Loaded {} categories from cache for month {}",
                cached.categories.len(),
                month
            );
            // Send cached data immediately
            let _ = self.data_tx.send(DataEvent::PlanCacheLoaded {
                month: cached.month.clone(),
                categories: cached.categories.clone(),
            });

            // Still fetch fresh data in background to keep cache updated
            self.fetch_plan_month(budget_id, month).await;
            return;
        }

        // Cache miss - fetch from API
        self.fetch_plan_month(budget_id, month).await;
    }

    /// Fetch plan data for a specific month from API
    async fn fetch_plan_month(&self, budget_id: String, month: String) {
        use ynab_api::endpoints::months::Month;
        let budget_id_api: BudgetId = budget_id.clone().into();
        let req = Request::months()
            .get()
            .budget_id(budget_id_api)
            .month(Month::Month(month.clone()));
        match self.api_client.send(req).await {
            Ok(response) => {
                tracing::info!(
                    "Loaded {} categories from API for month {}",
                    response.data.month.categories.len(),
                    month
                );
                // Send fresh data
                let _ = self.data_tx.send(DataEvent::PlanLoaded {
                    month: response.data.month.clone(),
                    categories: response.data.month.categories.clone(),
                });

                // Update cache in background
                let cache = self.cache.clone();
                let budget_id_clone = budget_id.clone();
                let month_clone = month.clone();
                let month_detail = response.data.month;
                tokio::spawn(async move {
                    let _ = cache
                        .set_plan_month(
                            &budget_id_clone,
                            &month_clone,
                            &month_detail,
                            &month_detail.categories,
                        )
                        .await;
                    tracing::debug!("Cached plan for month {} updated", month_clone);
                });
            }
            Err(e) => {
                tracing::error!("Failed to load plan for month {}: {}", month, e);
                let _ = self.data_tx.send(DataEvent::LoadError {
                    error: e.to_string(),
                });
            }
        }
    }

    /// Load payees for a budget (for transaction creation autocomplete)
    pub async fn load_payees(&self, budget_id: String, force_refresh: bool) {
        tracing::info!(
            "Loading payees for budget {} (force_refresh={})",
            budget_id,
            force_refresh
        );

        // Try cache first unless force refresh
        if !force_refresh {
            if let Ok(Some(cached)) = self.cache.get_payees(&budget_id).await {
                tracing::debug!("Loaded {} payees from cache", cached.len());
                let _ = self
                    .data_tx
                    .send(DataEvent::PayeesLoaded { payees: cached });
                return;
            }
        }

        // Load from API
        tracing::debug!("Fetching payees from API");
        let budget_id_api: BudgetId = budget_id.clone().into();
        let req = Request::payees().list().budget_id(budget_id_api);
        match self.api_client.send(req).await {
            Ok(response) => {
                // Filter out deleted payees
                let payees: Vec<_> = response
                    .data
                    .payees
                    .into_iter()
                    .filter(|p| !p.deleted)
                    .collect();

                tracing::info!("Loaded {} payees from API", payees.len());
                let _ = self.data_tx.send(DataEvent::PayeesLoaded {
                    payees: payees.clone(),
                });

                // Update cache in background
                let cache = self.cache.clone();
                let budget_id_clone = budget_id.clone();
                tokio::spawn(async move {
                    let _ = cache.set_payees(&budget_id_clone, &payees).await;
                    tracing::debug!("Cached payees updated");
                });
            }
            Err(e) => {
                tracing::error!("Failed to load payees from API: {}", e);
                let _ = self.data_tx.send(DataEvent::LoadError {
                    error: e.to_string(),
                });
            }
        }
    }

    /// Load categories for a budget (for transaction creation autocomplete)
    pub async fn load_categories(&self, budget_id: String, force_refresh: bool) {
        tracing::info!(
            "Loading categories for budget {} (force_refresh={})",
            budget_id,
            force_refresh
        );

        // Try cache first unless force refresh
        if !force_refresh {
            if let Ok(Some(cached)) = self.cache.get_categories(&budget_id).await {
                tracing::debug!("Loaded {} categories from cache", cached.len());
                let _ = self
                    .data_tx
                    .send(DataEvent::CategoriesLoaded { categories: cached });
                return;
            }
        }

        // Load from API
        tracing::debug!("Fetching categories from API");
        let budget_id_api: BudgetId = budget_id.clone().into();
        let req = Request::categories().list().budget_id(budget_id_api);
        match self.api_client.send(req).await {
            Ok(response) => {
                // Flatten category groups into single list with group name prefix
                let mut categories = Vec::new();
                for group in response.data.category_groups {
                    if !group.deleted && !group.hidden {
                        for mut category in group.categories {
                            if !category.deleted && !category.hidden {
                                // Set category_group_name for display
                                category.category_group_name = Some(group.name.clone());
                                categories.push(category);
                            }
                        }
                    }
                }

                tracing::info!("Loaded {} categories from API", categories.len());
                let _ = self.data_tx.send(DataEvent::CategoriesLoaded {
                    categories: categories.clone(),
                });

                // Update cache in background
                let cache = self.cache.clone();
                let budget_id_clone = budget_id.clone();
                tokio::spawn(async move {
                    let _ = cache.set_categories(&budget_id_clone, &categories).await;
                    tracing::debug!("Cached categories updated");
                });
            }
            Err(e) => {
                tracing::error!("Failed to load categories from API: {}", e);
                let _ = self.data_tx.send(DataEvent::LoadError {
                    error: e.to_string(),
                });
            }
        }
    }

    /// Create a new transaction
    pub async fn create_transaction(&self, budget_id: String, new_transaction: NewTransaction) {
        tracing::info!(
            "Creating transaction for account {} in budget {}",
            new_transaction.account_id,
            budget_id
        );

        let account_id = new_transaction.account_id;
        let budget_id_api: BudgetId = budget_id.clone().into();
        let mut req = Request::transactions().with_budget(budget_id_api).create(
            new_transaction.account_id,
            new_transaction.date,
            new_transaction.amount.inner(),
        );

        // Apply optional fields
        if let Some(payee_id) = new_transaction.payee_id {
            req = req.payee_id(payee_id);
        }
        if let Some(payee_name) = new_transaction.payee_name {
            req = req.payee_name(payee_name);
        }
        if let Some(category_id) = new_transaction.category_id {
            req = req.category_id(category_id);
        }
        if let Some(memo) = new_transaction.memo {
            req = req.memo(memo);
        }
        if let Some(cleared) = new_transaction.cleared {
            req = req.cleared(cleared);
        }
        if let Some(approved) = new_transaction.approved {
            req = req.approved(approved);
        }
        if let Some(flag_color) = new_transaction.flag_color {
            req = req.flag_color(flag_color);
        }
        if let Some(subtransactions) = new_transaction.subtransactions {
            req = req.subtransactions(subtransactions);
        }

        match self.api_client.send(req).await {
            Ok(response) => {
                tracing::info!(
                    "Transaction created successfully: {}",
                    response.data.transaction.id
                );
                let _ = self.data_tx.send(DataEvent::TransactionCreated {
                    transaction: response.data.transaction,
                });

                // Invalidate transaction cache to force refresh
                let cache = self.cache.clone();
                let budget_id_clone = budget_id.clone();
                let account_id_str = account_id.to_string();
                tokio::spawn(async move {
                    let _ = cache
                        .invalidate_transactions(&budget_id_clone, &account_id_str)
                        .await;
                    tracing::debug!("Transaction cache invalidated");
                });
            }
            Err(e) => {
                tracing::error!("Failed to create transaction: {}", e);
                let _ = self.data_tx.send(DataEvent::TransactionCreateFailed {
                    error: e.to_string(),
                });
            }
        }
    }

    /// Update a transaction (full update with all fields)
    pub async fn update_transaction_full(
        &self,
        budget_id: String,
        transaction_id: String,
        update: TransactionUpdate,
    ) {
        tracing::info!(
            "Updating transaction {} in budget {}",
            transaction_id,
            budget_id
        );

        let account_id = update.account_id;
        let budget_id_api: BudgetId = budget_id.clone().into();
        let txn_id: TransactionId = transaction_id.parse().expect("invalid transaction id");

        let mut req = Request::transactions()
            .with_budget(budget_id_api)
            .update(txn_id);

        // Apply all update fields
        if let Some(account_id) = update.account_id {
            req = req.account_id(account_id);
        }
        if let Some(date) = update.date {
            req = req.date(date);
        }
        if let Some(amount) = update.amount {
            req = req.amount(amount);
        }
        if let Some(payee_id) = update.payee_id {
            req = req.payee_id(payee_id);
        }
        if let Some(payee_name) = update.payee_name {
            req = req.payee_name(payee_name);
        }
        if let Some(category_id) = update.category_id {
            req = req.category_id(category_id);
        }
        if let Some(memo) = update.memo {
            req = req.memo(memo);
        }
        if let Some(flag_color) = update.flag_color {
            req = req.flag_color(flag_color);
        }
        if let Some(cleared) = update.cleared {
            req = req.cleared(cleared);
        }
        if let Some(approved) = update.approved {
            req = req.approved(approved);
        }
        if let Some(subtransactions) = update.subtransactions {
            req = req.subtransactions(subtransactions);
        }

        match self.api_client.send(req).await {
            Ok(response) => {
                tracing::info!("Transaction {} updated successfully", transaction_id);
                let _ = self.data_tx.send(DataEvent::TransactionUpdatedFull {
                    transaction: response.data.transaction,
                });

                // Invalidate transaction cache if account changed
                if let Some(new_account_id) = account_id {
                    let cache = self.cache.clone();
                    let budget_id_clone = budget_id.clone();
                    let account_id_str = new_account_id.to_string();
                    tokio::spawn(async move {
                        let _ = cache
                            .invalidate_transactions(&budget_id_clone, &account_id_str)
                            .await;
                        tracing::debug!("Transaction cache invalidated");
                    });
                }
            }
            Err(e) => {
                tracing::error!("Failed to update transaction {}: {}", transaction_id, e);
                let _ = self.data_tx.send(DataEvent::TransactionUpdateFullFailed {
                    transaction_id,
                    error: e.to_string(),
                });
            }
        }
    }

    /// Update a category's budgeted amount for a specific month
    pub async fn update_category_budget(
        &self,
        budget_id: String,
        month: String,
        category_id: String,
        budgeted: i64,
        original_budgeted: i64,
    ) {
        tracing::info!(
            "Updating budget for category {} in month {} to {}",
            category_id,
            month,
            budgeted
        );

        let budget_id_api: BudgetId = budget_id.into();
        let category_uuid: Uuid = category_id.parse().expect("invalid category id");

        let req = Request::categories()
            .with_budget(budget_id_api)
            .update_month(category_uuid, month, budgeted.into());

        match self.api_client.send(req).await {
            Ok(response) => {
                tracing::info!("Category budget updated successfully");
                let _ = self.data_tx.send(DataEvent::CategoryBudgetUpdated {
                    category: response.data.category,
                });
            }
            Err(e) => {
                tracing::error!("Failed to update category budget: {}", e);
                let _ = self.data_tx.send(DataEvent::CategoryBudgetUpdateFailed {
                    category_id: category_uuid.to_string(),
                    original_budgeted,
                    new_budgeted: budgeted,
                    error: e.to_string(),
                });
            }
        }
    }
}
