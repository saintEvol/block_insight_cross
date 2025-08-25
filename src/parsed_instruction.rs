use crate::instructions::spl_token::TokenInstruction as SplTokenInstruction;
use crate::instructions::spl_token_2022::TokenInstruction as SplToken2022Instruction;
use crate::token_transfer_data::TokenTransferData;
use crate::utils::TransactionAccounts;
use solana_pubkey::Pubkey;
use solana_sdk::program_error::ProgramError;
use solana_sdk::system_instruction::SystemInstruction;
use solana_transaction_status_client_types::EncodedTransaction::LegacyBinary;
use solana_transaction_status_client_types::option_serializer::OptionSerializer;
use solana_transaction_status_client_types::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiCompiledInstruction, UiInstruction,
    UiMessage,
};
use std::borrow::Cow;
use std::ops::Deref;
use thiserror::Error;
use tracing::error;

// #[cfg_attr(feature = "serde-traits", derive(Serialize, Deserialize))]
// #[cfg_attr(
//     feature = "serde-traits",
//     serde(rename_all_fields = "camelCase", rename_all = "camelCase")
// )]
#[derive(Debug, Error)]
pub enum ParseInstructionDataError {
    #[error("program error: {0}")]
    ProgramError(#[from] ProgramError),
    #[error("FromBase58Error: {0}")]
    ParseBase58Error(#[from] bs58::decode::Error),
    #[error("parse pubkey error: {0}")]
    ParsePubkeyError(String),
    #[error("bincode error: {0}")]
    BincodeError(#[from] bincode::Error),
}

#[cfg_attr(feature = "serde-traits", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde-traits",
    serde(rename_all_fields = "camelCase", rename_all = "camelCase")
)]
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedInstructionData {
    System(SystemInstruction),
    SplToken(SplTokenInstruction),
    SplToken2022(SplToken2022Instruction),
    Error(String),
    Unknown,
}

impl ParsedInstructionData {}

pub enum InstructionDataFormat<'a> {
    Binary(&'a [u8]),
    Base58(&'a str),
    // Base64(&'a str),
}

#[allow(unused)]
pub enum InstructionProgramId<'a> {
    Pubkey(&'a Pubkey),
    Base58(&'a str),
}

impl ParsedInstructionData {
    pub fn parse(
        program_id: InstructionProgramId,
        data: InstructionDataFormat,
    ) -> Result<Self, ParseInstructionDataError> {
        let pubkey = match program_id {
            InstructionProgramId::Pubkey(p) => Cow::Borrowed(p),
            InstructionProgramId::Base58(s) => {
                // let b = s
                //     .from_base58()
                //     .map_err(|e| ParseInstructionDataError::ParseBase58Error(format!("pubkey: {s}, error:{e:?}")))?;
                let b = bs58::decode(s).into_vec()?;
                let pubkey = Pubkey::try_from(b).map_err(|_e| {
                    ParseInstructionDataError::ParsePubkeyError("parse pubkey error".to_string())
                })?;
                Cow::Owned(pubkey)
            }
        };
        let data = match data {
            InstructionDataFormat::Binary(b) => Cow::Borrowed(b),
            InstructionDataFormat::Base58(b) => {
                let b = bs58::decode(b).into_vec()?;
                // let b = b
                //     .from_base58()
                //     .map_err(|e| ParseInstructionDataError::FromBase58Error(format!("data: {b}, error:{e:?}")))?;
                // if b.len() < 4 {
                //     b.resize(4, 0);
                // }
                Cow::Owned(b)
            }
        };
        Self::do_parse(&pubkey, &data)
    }

    fn do_parse(program: &Pubkey, data: &[u8]) -> Result<Self, ParseInstructionDataError> {
        if program == &solana_sdk::system_program::id() {
            // solana_sdk::system_instruction::SystemInstruction::deserialize()
            let sys = bincode::deserialize::<SystemInstruction>(data)?;
            return Ok(Self::System(sys));
        };

        if program == &spl_token::id() {
            let instruction = spl_token::instruction::TokenInstruction::unpack(data)?;
            return Ok(ParsedInstructionData::SplToken(
                crate::instructions::spl_token::TokenInstruction::from(instruction),
            ));
        }

        if program == &spl_token_2022::id() {
            let instruction = spl_token_2022::instruction::TokenInstruction::unpack(data)?;
            return Ok(ParsedInstructionData::SplToken2022(
                crate::instructions::spl_token_2022::TokenInstruction::from(instruction),
            ));
        }

        Ok(ParsedInstructionData::Unknown)
    }
}

#[cfg_attr(feature = "serde-traits", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-traits", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub instruction_data: ParsedInstructionData,
    pub inner_instructions: Option<Vec<ParsedInstruction>>,
}

impl ParsedInstruction {
    pub fn get_token_transfer_data(&self) -> Option<TokenTransferData> {
        match &self.instruction_data {
            ParsedInstructionData::SplToken(t) => match t {
                SplTokenInstruction::Transfer { amount } => {
                    let data = TokenTransferData {
                        source: self.accounts[0],
                        destination: self.accounts[1],
                        signer: self.accounts[2],
                        amount: *amount,
                        mint: None,
                        decimal: None,
                    };
                    Some(data)
                }
                SplTokenInstruction::TransferChecked { amount, decimals } => {
                    let data = TokenTransferData {
                        source: self.accounts[0],
                        mint: Some(self.accounts[1]),
                        destination: self.accounts[2],
                        signer: self.accounts[3],
                        amount: *amount,
                        decimal: Some(*decimals),
                    };
                    Some(data)
                }
                _ => return None,
            },
            ParsedInstructionData::SplToken2022(t) => match t {
                #[allow(deprecated)]
                SplToken2022Instruction::Transfer { amount } => {
                    let data = TokenTransferData {
                        source: self.accounts[0],
                        destination: self.accounts[1],
                        signer: self.accounts[2],
                        amount: *amount,
                        mint: None,
                        decimal: None,
                    };
                    Some(data)
                }
                SplToken2022Instruction::TransferChecked { amount, decimals } => {
                    let data = TokenTransferData {
                        source: self.accounts[0],
                        mint: Some(self.accounts[1]),
                        destination: self.accounts[2],
                        signer: self.accounts[3],
                        amount: *amount,
                        decimal: Some(*decimals),
                    };
                    Some(data)
                }
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg_attr(feature = "serde-traits", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-traits", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedInstructionList(pub Vec<ParsedInstruction>);

impl ParsedInstructionList {
    pub fn as_slice(&self) -> &[ParsedInstruction] {
        self.0.as_slice()
    }
}

impl Deref for ParsedInstructionList {
    type Target = [ParsedInstruction];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&EncodedTransactionWithStatusMeta> for ParsedInstructionList {
    fn from(value: &EncodedTransactionWithStatusMeta) -> Self {
        let message = match value.transaction {
            EncodedTransaction::Json(ref t) => match &t.message {
                UiMessage::Parsed(_) => {
                    error!("错误的Message类型: Parsed");
                    return ParsedInstructionList(Vec::new());
                }
                UiMessage::Raw(raw) => raw,
            },
            LegacyBinary(_) => {
                error!("错误的交易类型: LegacyBinary");
                return ParsedInstructionList(Vec::new());
            }
            EncodedTransaction::Binary(_, _) => {
                error!("错误的交易类型: Binary");
                return ParsedInstructionList(Vec::new());
            }
            EncodedTransaction::Accounts(_) => {
                error!("错误的交易类型: Accounts");
                return ParsedInstructionList(Vec::new());
            }
        };
        let meta = if let Some(meta) = &value.meta {
            meta
        } else {
            error!("交易缺少meta数据");
            return ParsedInstructionList(Vec::new());
        };
        // let (writable, readonly): (Vec<_>, Vec<_>) =
        let mut writable = None;
        let mut readonly = None;
        let loaded = meta.loaded_addresses.as_ref();
        let account_keys = message.account_keys.as_slice();
        match loaded {
            OptionSerializer::Some(r) => {
                writable = Some(r.writable.as_slice());
                readonly = Some(r.readonly.as_slice());
            }
            _ => {}
        }

        let transaction_accounts =
            TransactionAccounts::from_accounts(Some(account_keys), writable, readonly);
        let raw_instructions = &message.instructions;

        let mut instructions = Vec::with_capacity(raw_instructions.len());
        for raw in raw_instructions {
            if let Some(parsed) = Self::parse_ui_compiled_instruction(&transaction_accounts, raw) {
                instructions.push(parsed);
            }
        }

        // 处理子指令
        if let OptionSerializer::Some(inner) = &meta.inner_instructions {
            // 内部指令组
            for inner_instructions in inner {
                if let Some(parent) = instructions.get_mut(inner_instructions.index as usize) {
                    // 单个内部指令
                    for inner_instruction in &inner_instructions.instructions {
                        match inner_instruction {
                            UiInstruction::Compiled(inner_instruction) => {
                                if let Some(parsed) = Self::parse_ui_compiled_instruction(
                                    &transaction_accounts,
                                    inner_instruction,
                                ) {
                                    if let Some(old) = parent.inner_instructions.as_mut() {
                                        old.push(parsed);
                                    } else {
                                        let mut inner_instructions = Vec::with_capacity(
                                            inner_instructions.instructions.len(),
                                        );
                                        inner_instructions.push(parsed);
                                        parent.inner_instructions = Some(inner_instructions);
                                    }
                                }
                            }
                            UiInstruction::Parsed(_) => {
                                error!("意外的内部指令格式：Parsed");
                            }
                        }
                    }
                }
            }
        }

        ParsedInstructionList(instructions)
    }
}

impl ParsedInstructionList {
    fn parse_ui_compiled_instruction(
        transaction_accounts: &TransactionAccounts<String>,
        compiled: &UiCompiledInstruction,
    ) -> Option<ParsedInstruction> {
        if let Some(program) = transaction_accounts.get(compiled.program_id_index as usize) {
            match ParsedInstructionData::parse(
                InstructionProgramId::Base58(program.as_str()),
                InstructionDataFormat::Base58(&compiled.data),
            ) {
                Ok(parsed) => Some(ParsedInstruction {
                    program_id_index: compiled.program_id_index,
                    accounts: compiled.accounts.clone(),
                    instruction_data: parsed,
                    inner_instructions: None,
                }),
                Err(e) => {
                    error!("解析指令数据出错: {e:?}");
                    Some(ParsedInstruction {
                        program_id_index: compiled.program_id_index,
                        accounts: compiled.accounts.clone(),
                        instruction_data: ParsedInstructionData::Error(format!("{}", e)),
                        inner_instructions: None,
                    })
                }
            }
        } else {
            error!("无法获取指令所属程序");
            None
        }
    }
}
