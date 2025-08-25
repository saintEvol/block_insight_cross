#[derive(Debug)]
pub struct TokenTransferData {
    pub source: u8,
    pub destination: u8,
    pub signer: u8,
    pub amount: u64,
    // TransferCheck才有
    pub mint: Option<u8>,
    // TransferCheck才有
    pub decimal: Option<u8>,
}
