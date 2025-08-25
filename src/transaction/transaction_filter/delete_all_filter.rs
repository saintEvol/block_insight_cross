use crate::transaction::transaction_filter::{
    TransactionFilter, TransactionFilterContext, TransactionPropsProvider,
};

pub struct DeleteAllFilter;

impl TransactionFilter for DeleteAllFilter {
    type ContextType = TransactionFilterContext;

    fn filter(
        &self,
        _obj: &dyn TransactionPropsProvider,
        _context: &mut Self::ContextType,
    ) -> bool {
        false
    }
}
