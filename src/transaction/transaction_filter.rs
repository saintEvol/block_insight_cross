pub mod account_filter;
pub mod delete_all_filter;
pub mod signature_filter;
pub mod circle_swap_filter;
pub mod status_filter;

use std::any::TypeId;
use solana_sdk::transaction::TransactionError;
use solana_transaction_error::TransactionResult;
use crate::parsed_instruction::ParsedInstruction;
use crate::utils::TransactionAccounts;

#[derive(Default)]
pub struct TransactionFilterContext {}

pub struct TransactionMeta<'a> {
    pub err: Option<&'a TransactionError>,
    pub status: &'a TransactionResult<()>, // This field is deprecated.  See https://github.com/solana-labs/solana/issues/9302
    pub fee: u64,
    pub pre_balances: &'a [u64],
    pub post_balances: &'a [u64],
    pub log_messages: Option<&'a [String]>,
    pub compute_units_consumed: Option<u64>,
}

pub trait TransactionPropsProvider {
    fn get_accounts(&self) -> TransactionAccounts<'_, String>;

    /// 获取交易签名，如果交易格式不对，可能导致无法获取到签名，此时返回None
    fn get_signatures(&self) -> Option<&[String]>;

    fn get_parsed_instructions(&self) -> Option<&[ParsedInstruction]>;
    fn get_meta(&self) -> Option<TransactionMeta<'_>>;
}

pub trait TransactionFilter: Send + Sync + 'static {
    type ContextType;

    fn filter(&self, _obj: &dyn TransactionPropsProvider, _context: &mut Self::ContextType) -> bool {
        true
    }

    fn id(&self) -> TypeId {
        std::any::TypeId::of::<Self>()
    }
}

