use crate::transaction::transaction_filter::{
    TransactionFilter, TransactionFilterContext, TransactionPropsProvider,
};

pub struct TransactionStatusFilter {
    is_success: bool,
}

impl TransactionStatusFilter {
    pub fn success() -> Self {
        Self { is_success: true }
    }

    pub fn fail() -> Self {
        Self { is_success: false }
    }
}

impl TransactionFilter for TransactionStatusFilter {
    type ContextType = TransactionFilterContext;

    fn filter(&self, obj: &dyn TransactionPropsProvider, _context: &mut Self::ContextType) -> bool {
        let meta = obj.get_meta();
        if let Some(meta) = meta {
            meta.status.is_ok() == self.is_success
        } else {
            false
        }
    }
}
