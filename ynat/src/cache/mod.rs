use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use ynab_api::endpoints::{
    accounts::Account, budgets::BudgetSummary, categories::Category, months::MonthDetail,
    payees::Payee, transactions::Transaction,
};

#[derive(Debug)]
pub enum CacheError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Io(e) => write!(f, "IO error: {}", e),
            CacheError::Serialization(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<std::io::Error> for CacheError {
    fn from(err: std::io::Error) -> Self {
        CacheError::Io(err)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::Serialization(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBudgets {
    pub budgets: Vec<BudgetSummary>,
    pub default_budget: Option<BudgetSummary>,
    pub cached_at: i64, // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAccounts {
    pub accounts: Vec<Account>,
    pub server_knowledge: Option<i64>,
    pub cached_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTransactions {
    pub transactions: Vec<Transaction>,
    pub server_knowledge: Option<i64>,
    pub cached_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPlan {
    pub month: MonthDetail,
    pub categories: Vec<Category>,
    pub server_knowledge: Option<i64>,
    pub cached_at: i64,
}

/// Async cache layer using tokio::fs for non-blocking file I/O
#[derive(Clone)]
pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub async fn new() -> Result<Self, CacheError> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir).await?;

        Ok(Self { cache_dir })
    }

    fn get_cache_dir() -> Result<PathBuf, CacheError> {
        let cache_dir = dirs::cache_dir()
            .expect("Always returns")
            .join("ynat")
            .join("data");

        Ok(cache_dir)
    }

    // Budgets cache
    pub async fn get_budgets(&self) -> Result<Option<CachedBudgets>, CacheError> {
        let path = self.cache_dir.join("budgets.json");
        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&path).await?;
        let cached: CachedBudgets = serde_json::from_str(&data)?;
        Ok(Some(cached))
    }

    pub async fn set_budgets(
        &self,
        budgets: &[BudgetSummary],
        default_budget: Option<BudgetSummary>,
    ) -> Result<(), CacheError> {
        let cached = CachedBudgets {
            budgets: budgets.to_vec(),
            default_budget,
            cached_at: chrono::Utc::now().timestamp(),
        };

        let path = self.cache_dir.join("budgets.json");
        let json = serde_json::to_string_pretty(&cached)?;
        fs::write(&path, json).await?;
        Ok(())
    }

    // Accounts cache
    pub async fn get_accounts(
        &self,
        budget_id: &str,
    ) -> Result<Option<CachedAccounts>, CacheError> {
        let path = self.cache_dir.join(format!("accounts_{}.json", budget_id));
        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&path).await?;
        let cached: CachedAccounts = serde_json::from_str(&data)?;
        Ok(Some(cached))
    }

    pub async fn set_accounts(
        &self,
        budget_id: &str,
        accounts: &[Account],
        server_knowledge: Option<i64>,
    ) -> Result<(), CacheError> {
        let cached = CachedAccounts {
            accounts: accounts.to_vec(),
            server_knowledge,
            cached_at: chrono::Utc::now().timestamp(),
        };

        let path = self.cache_dir.join(format!("accounts_{}.json", budget_id));
        let json = serde_json::to_string_pretty(&cached)?;
        fs::write(&path, json).await?;
        Ok(())
    }

    /// Merge delta updates into existing accounts cache
    pub async fn merge_accounts_delta(
        &self,
        budget_id: &str,
        delta: &[Account],
        new_server_knowledge: i64,
    ) -> Result<(), CacheError> {
        // Read existing cache
        let mut cached = self.get_accounts(budget_id).await?.ok_or_else(|| {
            CacheError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cache not found for merge",
            ))
        })?;

        // Merge delta
        for delta_account in delta {
            if delta_account.deleted {
                cached.accounts.retain(|a| a.id != delta_account.id);
            } else if let Some(existing) = cached
                .accounts
                .iter_mut()
                .find(|a| a.id == delta_account.id)
            {
                *existing = delta_account.clone();
            } else {
                cached.accounts.push(delta_account.clone());
            }
        }

        cached.server_knowledge = Some(new_server_knowledge);
        cached.cached_at = chrono::Utc::now().timestamp();

        // Write back
        self.set_accounts(budget_id, &cached.accounts, cached.server_knowledge)
            .await
    }

    // Transactions cache
    pub async fn get_transactions(
        &self,
        budget_id: &str,
        account_id: &str,
    ) -> Result<Option<CachedTransactions>, CacheError> {
        let path = self
            .cache_dir
            .join(format!("transactions_{}_{}.json", budget_id, account_id));
        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&path).await?;
        let cached: CachedTransactions = serde_json::from_str(&data)?;
        Ok(Some(cached))
    }

    pub async fn set_transactions(
        &self,
        budget_id: &str,
        account_id: &str,
        transactions: &[Transaction],
        server_knowledge: Option<i64>,
    ) -> Result<(), CacheError> {
        let cached = CachedTransactions {
            transactions: transactions.to_vec(),
            server_knowledge,
            cached_at: chrono::Utc::now().timestamp(),
        };

        let path = self
            .cache_dir
            .join(format!("transactions_{}_{}.json", budget_id, account_id));
        let json = serde_json::to_string_pretty(&cached)?;
        fs::write(&path, json).await?;
        Ok(())
    }

    /// Merge delta updates into existing transactions cache
    pub async fn merge_transactions_delta(
        &self,
        budget_id: &str,
        account_id: &str,
        delta: &[Transaction],
        new_server_knowledge: i64,
    ) -> Result<(), CacheError> {
        // Read existing cache
        let mut cached = self
            .get_transactions(budget_id, account_id)
            .await?
            .ok_or_else(|| {
                CacheError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Cache not found for merge",
                ))
            })?;

        // Merge delta
        for delta_transaction in delta {
            if delta_transaction.deleted {
                cached.transactions.retain(|t| t.id != delta_transaction.id);
            } else if let Some(existing) = cached
                .transactions
                .iter_mut()
                .find(|t| t.id == delta_transaction.id)
            {
                *existing = delta_transaction.clone();
            } else {
                cached.transactions.push(delta_transaction.clone());
            }
        }

        cached.server_knowledge = Some(new_server_knowledge);
        cached.cached_at = chrono::Utc::now().timestamp();

        // Write back
        self.set_transactions(
            budget_id,
            account_id,
            &cached.transactions,
            cached.server_knowledge,
        )
        .await
    }

    // Plan cache
    pub async fn get_plan(&self, budget_id: &str) -> Result<Option<CachedPlan>, CacheError> {
        let path = self.cache_dir.join(format!("plan_{}.json", budget_id));
        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&path).await?;
        let cached: CachedPlan = serde_json::from_str(&data)?;
        Ok(Some(cached))
    }

    pub async fn set_plan(
        &self,
        budget_id: &str,
        month: &MonthDetail,
        categories: &[Category],
        server_knowledge: Option<i64>,
    ) -> Result<(), CacheError> {
        let cached = CachedPlan {
            month: month.clone(),
            categories: categories.to_vec(),
            server_knowledge,
            cached_at: chrono::Utc::now().timestamp(),
        };

        let path = self.cache_dir.join(format!("plan_{}.json", budget_id));
        let json = serde_json::to_string_pretty(&cached)?;
        fs::write(&path, json).await?;
        Ok(())
    }

    /// Get plan for a specific month (month format: YYYY-MM-DD)
    pub async fn get_plan_month(
        &self,
        budget_id: &str,
        month: &str,
    ) -> Result<Option<CachedPlan>, CacheError> {
        let path = self
            .cache_dir
            .join(format!("plan_{}_{}.json", budget_id, month));
        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&path).await?;
        let cached: CachedPlan = serde_json::from_str(&data)?;
        Ok(Some(cached))
    }

    /// Set plan for a specific month (month format: YYYY-MM-DD)
    pub async fn set_plan_month(
        &self,
        budget_id: &str,
        month_str: &str,
        month: &MonthDetail,
        categories: &[Category],
    ) -> Result<(), CacheError> {
        let cached = CachedPlan {
            month: month.clone(),
            categories: categories.to_vec(),
            server_knowledge: None,
            cached_at: chrono::Utc::now().timestamp(),
        };

        let path = self
            .cache_dir
            .join(format!("plan_{}_{}.json", budget_id, month_str));
        let json = serde_json::to_string_pretty(&cached)?;
        fs::write(&path, json).await?;
        Ok(())
    }

    // Payees cache (for transaction creation autocomplete)
    pub async fn get_payees(&self, budget_id: &str) -> Result<Option<Vec<Payee>>, CacheError> {
        let path = self.cache_dir.join(format!("payees_{}.json", budget_id));
        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&path).await?;
        let payees: Vec<Payee> = serde_json::from_str(&data)?;
        Ok(Some(payees))
    }

    pub async fn set_payees(&self, budget_id: &str, payees: &[Payee]) -> Result<(), CacheError> {
        let path = self.cache_dir.join(format!("payees_{}.json", budget_id));
        let json = serde_json::to_string_pretty(&payees)?;
        fs::write(&path, json).await?;
        Ok(())
    }

    // Categories cache (for transaction creation autocomplete)
    pub async fn get_categories(
        &self,
        budget_id: &str,
    ) -> Result<Option<Vec<Category>>, CacheError> {
        let path = self
            .cache_dir
            .join(format!("categories_{}.json", budget_id));
        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&path).await?;
        let categories: Vec<Category> = serde_json::from_str(&data)?;
        Ok(Some(categories))
    }

    pub async fn set_categories(
        &self,
        budget_id: &str,
        categories: &[Category],
    ) -> Result<(), CacheError> {
        let path = self
            .cache_dir
            .join(format!("categories_{}.json", budget_id));
        let json = serde_json::to_string_pretty(&categories)?;
        fs::write(&path, json).await?;
        Ok(())
    }

    // Invalidate transactions cache (after creating a new transaction)
    pub async fn invalidate_transactions(
        &self,
        budget_id: &str,
        account_id: &str,
    ) -> Result<(), CacheError> {
        let path = self
            .cache_dir
            .join(format!("transactions_{}_{}.json", budget_id, account_id));

        if path.exists() {
            fs::remove_file(&path).await?;
            tracing::debug!(
                "Invalidated transactions cache for budget {} account {}",
                budget_id,
                account_id
            );
        }

        Ok(())
    }

    // Invalidate plan cache (after updating a category budget)
    pub async fn invalidate_plan(&self, budget_id: &str) -> Result<(), CacheError> {
        let path = self.cache_dir.join(format!("plan_{}.json", budget_id));

        if path.exists() {
            fs::remove_file(&path).await?;
            tracing::debug!("Invalidated plan cache for budget {}", budget_id);
        }

        Ok(())
    }
}
