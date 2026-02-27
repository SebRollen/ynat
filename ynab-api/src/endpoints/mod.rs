pub mod accounts;
pub mod budgets;
pub mod categories;
pub mod months;
pub mod payees;
pub mod transactions;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetId {
    #[serde(rename = "last-used")]
    LastUsed,
    #[default]
    #[serde(rename = "default")]
    Default,
    #[serde(untagged)]
    Uuid(Uuid),
}

impl Display for BudgetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LastUsed => f.write_str("last-used"),
            Self::Default => f.write_str("default"),
            Self::Uuid(uuid) => uuid.fmt(f),
        }
    }
}

impl From<Uuid> for BudgetId {
    fn from(uuid: Uuid) -> Self {
        BudgetId::Uuid(uuid)
    }
}

impl From<&str> for BudgetId {
    fn from(s: &str) -> Self {
        match s {
            "last-used" => BudgetId::LastUsed,
            "default" => BudgetId::Default,
            _ => BudgetId::Uuid(Uuid::parse_str(s).expect("invalid uuid for BudgetId")),
        }
    }
}

impl From<String> for BudgetId {
    fn from(s: String) -> Self {
        BudgetId::from(s.as_str())
    }
}

impl PartialEq<str> for BudgetId {
    fn eq(&self, other: &str) -> bool {
        self.to_string() == other
    }
}

impl PartialEq<&str> for BudgetId {
    fn eq(&self, other: &&str) -> bool {
        self.to_string() == *other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct LastKnowledgeOfServer(i64);

impl From<i64> for LastKnowledgeOfServer {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<LastKnowledgeOfServer> for i64 {
    fn from(value: LastKnowledgeOfServer) -> Self {
        value.0
    }
}

impl LastKnowledgeOfServer {
    pub fn inner(&self) -> i64 {
        self.0
    }
}

/// Query wrapper for LastKnowledgeOfServer that serializes with the field name
#[derive(Debug, Clone, Serialize)]
pub struct LastKnowledgeQuery {
    pub last_knowledge_of_server: i64,
}

impl From<&LastKnowledgeOfServer> for LastKnowledgeQuery {
    fn from(value: &LastKnowledgeOfServer) -> Self {
        Self {
            last_knowledge_of_server: value.0,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Milliunits(i64);

impl Milliunits {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> i64 {
        self.0
    }

    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }

    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }
}

impl From<i64> for Milliunits {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<Milliunits> for i64 {
    fn from(value: Milliunits) -> Self {
        value.0
    }
}

impl std::ops::Add for Milliunits {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for Milliunits {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub for Milliunits {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign for Milliunits {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::iter::Sum for Milliunits {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(0), |acc, x| acc + x)
    }
}

impl std::fmt::Display for Milliunits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateFormat {
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrencyFormat {
    pub iso_code: String,
    pub example_format: String,
    pub decimal_digits: i32,
    pub decimal_separator: String,
    pub symbol_first: bool,
    pub group_separator: String,
    pub currency_symbol: String,
    pub display_symbol: bool,
}

/// Transaction ID type that represents either a plain UUID or a UUID with a date suffix.
///
/// YNAB transaction IDs can be in three formats:
/// - Plain UUID: `04ce130f-5b1d-4328-895d-8abfa094a62b`
/// - UUID with date suffix: `04ce130f-5b1d-4328-895d-8abfa094a62b_2019-07-31`
/// - Transfer UUID with date suffix: `04ce130f-5b1d-4328-895d-8abfa094a62b_t_2019-07-31`
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionId {
    uuid: Uuid,
    suffix: Option<TransactionIdSuffix>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TransactionIdSuffix {
    /// Regular date suffix: `_{date}`
    Date(NaiveDate),
    /// Transfer date suffix: `_t_{date}`
    Transfer(NaiveDate),
}

impl TransactionId {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid, suffix: None }
    }

    pub fn with_date(uuid: Uuid, date: NaiveDate) -> Self {
        Self {
            uuid,
            suffix: Some(TransactionIdSuffix::Date(date)),
        }
    }

    pub fn with_transfer_date(uuid: Uuid, date: NaiveDate) -> Self {
        Self {
            uuid,
            suffix: Some(TransactionIdSuffix::Transfer(date)),
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn date_suffix(&self) -> Option<NaiveDate> {
        match &self.suffix {
            Some(TransactionIdSuffix::Date(date)) => Some(*date),
            Some(TransactionIdSuffix::Transfer(date)) => Some(*date),
            None => None,
        }
    }

    pub fn is_transfer(&self) -> bool {
        matches!(self.suffix, Some(TransactionIdSuffix::Transfer(_)))
    }
}

impl Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.suffix {
            Some(TransactionIdSuffix::Date(date)) => write!(f, "{}_{}", self.uuid, date),
            Some(TransactionIdSuffix::Transfer(date)) => write!(f, "{}_t_{}", self.uuid, date),
            None => write!(f, "{}", self.uuid),
        }
    }
}

impl FromStr for TransactionId {
    type Err = TransactionIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try parsing as UUID_t_date format (transfer)
        if let Some((prefix, date_part)) = s.rsplit_once("_t_") {
            if let Ok(uuid) = Uuid::parse_str(prefix) {
                if let Ok(date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                    return Ok(Self::with_transfer_date(uuid, date));
                }
            }
        }

        // Try parsing as UUID_date format
        if let Some((uuid_part, date_part)) = s.rsplit_once('_') {
            if let Ok(uuid) = Uuid::parse_str(uuid_part) {
                if let Ok(date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                    return Ok(Self::with_date(uuid, date));
                }
            }
        }

        // Fall back to plain UUID
        match Uuid::parse_str(s) {
            Ok(uuid) => Ok(Self::new(uuid)),
            Err(_) => Err(TransactionIdParseError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionIdParseError(String);

impl std::fmt::Display for TransactionIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid transaction ID '{}': expected UUID, UUID_YYYY-MM-DD, or UUID_t_YYYY-MM-DD format",
            self.0
        )
    }
}

impl std::error::Error for TransactionIdParseError {}

impl From<Uuid> for TransactionId {
    fn from(uuid: Uuid) -> Self {
        Self::new(uuid)
    }
}

impl Serialize for TransactionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TransactionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
