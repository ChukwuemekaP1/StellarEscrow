use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trade {
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub arbitrator: Option<Address>,
    pub status: TradeStatus,
    pub created_at: u32,
    pub updated_at: u32,
    pub metadata: Option<TradeMetadata>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionRecord {
    pub trade_id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub status: TradeStatus,
    pub created_at: u32,
    pub updated_at: u32,
    pub metadata: Option<TradeMetadata>,
}

pub const METADATA_MAX_VALUE_LEN: u32 = 256;
pub const METADATA_MAX_ENTRIES: u32 = 10;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeMetadata {
    pub entries: Vec<MetadataEntry>,
}

// ---------------------------------------------------------------------------
// Fee Tier System
// ---------------------------------------------------------------------------

pub const TIER_SILVER_THRESHOLD: u64 = 10_000_000_000;
pub const TIER_GOLD_THRESHOLD: u64 = 100_000_000_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserTier {
    Bronze,
    Silver,
    Gold,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserTierInfo {
    pub tier: UserTier,
    pub total_volume: u64,
    pub custom_fee_bps: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    pub bronze_fee_bps: u32,
    pub silver_fee_bps: u32,
    pub gold_fee_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryFilter {
    pub status: Option<TradeStatus>,
    pub from_ledger: Option<u32>,
    pub to_ledger: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryPage {
    pub records: Vec<TransactionRecord>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

// ---------------------------------------------------------------------------
// Trade Templates
// ---------------------------------------------------------------------------

pub const TEMPLATE_NAME_MAX_LEN: u32 = 64;
pub const TEMPLATE_MAX_VERSIONS: u32 = 10;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateTerms {
    pub description: String,
    pub default_arbitrator: Option<Address>,
    pub fixed_amount: Option<u64>,
    pub default_metadata: Option<TradeMetadata>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateVersion {
    pub version: u32,
    pub terms: TemplateTerms,
    pub created_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeTemplate {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub current_version: u32,
    pub versions: Vec<TemplateVersion>,
    pub active: bool,
    pub created_at: u32,
    pub updated_at: u32,
}

// ---------------------------------------------------------------------------
// User Management
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    pub username_hash: soroban_sdk::Bytes,
    pub contact_hash: soroban_sdk::Bytes,
    pub verification: VerificationStatus,
    pub registered_at: u32,
    pub updated_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPreference {
    pub key: soroban_sdk::String,
    pub value: soroban_sdk::String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserAnalytics {
    pub address: Address,
    pub total_trades: u32,
    pub trades_as_seller: u32,
    pub trades_as_buyer: u32,
    pub total_volume: u64,
    pub completed_trades: u32,
    pub disputed_trades: u32,
    pub cancelled_trades: u32,
}

// ---------------------------------------------------------------------------
// Admin Panel
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformAnalytics {
    pub total_trades: u64,
    pub total_volume: u64,
    pub total_fees_collected: u64,
    pub active_trades: u64,
    pub completed_trades: u64,
    pub disputed_trades: u64,
    pub cancelled_trades: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemConfig {
    pub fee_bps: u32,
    pub is_paused: bool,
    pub trade_counter: u64,
    pub accumulated_fees: u64,
}

// ---------------------------------------------------------------------------
// Trade Detail View
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineEntry {
    pub status: TradeStatus,
    pub ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeAction {
    Fund,
    Complete,
    ConfirmReceipt,
    RaiseDispute,
    Cancel,
    ResolveDispute,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeDetail {
    pub trade: Trade,
    pub timeline: Vec<TimelineEntry>,
    pub available_actions: Vec<TradeAction>,
    pub seller_payout: u64,
}

// ---------------------------------------------------------------------------
// Advanced Filtering & Sorting
// ---------------------------------------------------------------------------

/// Field to sort trades by.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeSortField {
    CreatedAt,
    UpdatedAt,
    Amount,
    Fee,
}

/// A single sort criterion: field + direction.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortCriterion {
    pub field: TradeSortField,
    pub order: SortOrder,
}

/// Multi-criteria filter for advanced trade search.
/// All set fields are ANDed together.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeFilter {
    /// Filter by trade status
    pub status: Option<TradeStatus>,
    /// Minimum trade amount (inclusive)
    pub min_amount: Option<u64>,
    /// Maximum trade amount (inclusive)
    pub max_amount: Option<u64>,
    /// Minimum created_at ledger (inclusive)
    pub from_ledger: Option<u32>,
    /// Maximum created_at ledger (inclusive)
    pub to_ledger: Option<u32>,
    /// Only return trades where this address is seller
    pub seller: Option<Address>,
    /// Only return trades where this address is buyer
    pub buyer: Option<Address>,
}

/// Paginated result for advanced trade search.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeSearchPage {
    pub records: Vec<TransactionRecord>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

/// Maximum number of presets a user can save.
pub const MAX_PRESETS_PER_USER: u32 = 20;
/// Maximum length of a preset name.
pub const PRESET_NAME_MAX_LEN: u32 = 64;

/// A saved filter preset owned by a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterPreset {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub filter: TradeFilter,
    pub sort: SortCriterion,
    pub created_at: u32,
    pub updated_at: u32,
}
