use crate::transaction::transaction_filter::{
    TransactionFilter, TransactionFilterContext, TransactionPropsProvider,
};

#[derive(Debug)]
pub enum SignatureFilter {
    Include(Vec<String>),
    Exclude(Vec<String>),
}

impl TransactionFilter for SignatureFilter {
    type ContextType = TransactionFilterContext;

    fn filter(&self, obj: &dyn TransactionPropsProvider, _context: &mut Self::ContextType) -> bool {
        let signatures = obj.get_signatures();

        match self {
            SignatureFilter::Include(including_signatures) => signatures
                .map(|s| s.iter().any(|s| including_signatures.contains(s)))
                .unwrap_or(true),
            SignatureFilter::Exclude(excluding_signatures) => !signatures
                .map(|s| s.iter().any(|s| excluding_signatures.contains(s)))
                .unwrap_or(false),
        }
    }
}
