#![no_std]

mod admin;
mod errors;
mod events;
mod filtering;
mod history;
mod storage;
mod templates;
mod tiers;
mod trade_detail;
mod types;
mod users;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use soroban_sdk::token::TokenClient;

use types::{METADATA_MAX_ENTRIES, METADATA_MAX_VALUE_LEN};

pub use errors::ContractError;
pub use types::{
    DisputeResolution, FilterPreset, HistoryFilter, HistoryPage, MetadataEntry,
    PlatformAnalytics, SortCriterion, SortOrder, SystemConfig, TierConfig, Trade,
    TradeDetail, TradeFilter, TradeMetadata, TradeSearchPage, TradeStatus, TradeTemplate,
    TemplateTerms, TemplateVersion, TradeSortField, TransactionRecord, UserAnalytics,
    UserProfile, UserPreference, UserTier, UserTierInfo, VerificationStatus,
};

use storage::{
    append_timeline_entry, get_accumulated_fees, get_admin, get_fee_bps, get_trade,
    get_usdc_token, has_arbitrator, increment_trade_counter, index_trade_for_address,
    is_initialized, is_paused, remove_arbitrator, save_arbitrator, save_trade,
    set_accumulated_fees, set_admin, set_fee_bps, set_initialized, set_paused,
    set_trade_counter, set_usdc_token,
};

use types::TimelineEntry;

fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}

fn validate_metadata(meta: &TradeMetadata) -> Result<(), ContractError> {
    if meta.entries.len() > METADATA_MAX_ENTRIES {
        return Err(ContractError::MetadataTooManyEntries);
    }
    for entry in meta.entries.iter() {
        if entry.value.len() > METADATA_MAX_VALUE_LEN {
            return Err(ContractError::MetadataValueTooLong);
        }
    }
    Ok(())
}
