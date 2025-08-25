use crate::transaction::transaction_filter::{
    TransactionFilter, TransactionFilterContext, TransactionPropsProvider,
};
use std::collections::HashSet;
use crate::parsed_instruction::ParsedInstruction;
use crate::token_transfer_data::TokenTransferData;
use crate::utils::TransactionAccounts;

pub struct CircleSwapFilter;

impl TransactionFilter for CircleSwapFilter {
    type ContextType = TransactionFilterContext;

    fn filter(&self, obj: &dyn TransactionPropsProvider, _context: &mut Self::ContextType) -> bool {
        // info!("现在过滤三角套利");
        let accounts = obj.get_accounts();
        let mut source: HashSet<String> = HashSet::new();
        if let Some(parsed_instructions) = obj.get_parsed_instructions() {
            for ins in parsed_instructions {
                if Self::handle_instruction(ins, &mut source, &accounts) {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }
}

impl CircleSwapFilter {
    /// 处理指令，检查是否有三角指令，如果有返回true
    fn handle_instruction<'a>(
        parsed_instruction: &ParsedInstruction,
        source: &mut HashSet<String>,
        accounts: &TransactionAccounts<'a, String>,
    ) -> bool {
        if let Some(transfer_data) = parsed_instruction.get_token_transfer_data() {
            return Self::handle_token_transfer_data(source, &accounts, &transfer_data);
        }

        // 指令不是Transfer相关指令，检查子命令列表
        // 检查子指令
        if let Some(inner) = parsed_instruction.inner_instructions.as_ref() {
            for inner_instruction in inner {
                if Self::handle_instruction(inner_instruction, source, accounts) {
                    return true;
                }
            }
        }

        false
    }

    /// 根据TokenTransferData检查是否存在三角套利，如果是返回true
    fn handle_token_transfer_data<'a>(
        source: &mut HashSet<String>,
        accounts: &TransactionAccounts<'a, String>,
        transfer_data: &TokenTransferData,
    ) -> bool {
        if let Some(destination) = accounts.get(transfer_data.destination as usize) {
            if source.contains(destination) {
                return true;
            } else {
                if let Some(s) = accounts.get(transfer_data.source as usize) {
                    source.insert(s.clone());
                }
            }
        }

        false
    }
}
