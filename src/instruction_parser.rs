use crate::parsed_instruction::{ParsedInstructionData};
use solana_pubkey::Pubkey;

pub trait InstructionParser {
    type ParseError;

    fn parse(&self, program_id: Pubkey) -> Option<Result<ParsedInstructionData, Self::ParseError>>;
}
