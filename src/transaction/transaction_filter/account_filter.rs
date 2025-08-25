use crate::transaction::transaction_filter::{
    TransactionFilter, TransactionFilterContext, TransactionPropsProvider,
};

/// 帐号过滤器，用来过滤交易中的帐号
pub enum AccountFilter {
    Include(Vec<String>),
    Exclude(Vec<String>),
}

impl AccountFilter {
    pub fn include(accounts: Vec<String>) -> Self {
        Self::Include(accounts)
    }

    pub fn exclude(accounts: Vec<String>) -> Self {
        Self::Exclude(accounts)
    }
}

impl TransactionFilter for AccountFilter {
    type ContextType = TransactionFilterContext;

    fn filter(&self, obj: &dyn TransactionPropsProvider, _context: &mut Self::ContextType) -> bool {
        let transaction_accounts = obj.get_accounts();
        match self {
            AccountFilter::Include(accounts) => {
                accounts.iter().all(|s| transaction_accounts.contains(s))
            }
            AccountFilter::Exclude(accounts) => {
                accounts.iter().all(|s| !transaction_accounts.contains(s))
            }
        }
    }
}
