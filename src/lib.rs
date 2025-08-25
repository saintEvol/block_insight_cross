
pub mod instructions;
pub mod parsed_instruction;
pub mod utils;
pub mod instruction_parser;
pub mod pubkeys;
pub mod token_transfer_data;
pub mod transaction;
// fn parse_tip(
//     transaction_accounts: &[Vec<u8>],
//     instruction: &CompiledInstruction,
// ) -> Option<(TransactionChannelType, u64)> {
//     if let Some(program) = transaction_accounts.get(instruction.program_id_index as usize) {
//         let program = match Pubkey::try_from_slice(program) {
//             Ok(p) => p,
//             Err(e) => {
//                 error!("解析程序id出错:{e:?}");
//                 return None;
//             }
//         };
//         if program == solana_sdk::system_program::id() {
//             match bincode::deserialize::<SystemInstruction>(&instruction.data) {
//                 Ok(inst) => match inst {
//                     SystemInstruction::Transfer { lamports } => {
//                         if let Some(recipient) = Self::collect_system_transfer_recipient_account(
//                             &transaction_accounts,
//                             &instruction.accounts,
//                         ) {
//                             Some((Self::get_channel_type(recipient), lamports))
//                         } else {
//                             None
//                         }
//                     }
//                     SystemInstruction::TransferWithSeed { lamports, .. } => {
//                         if let Some(recipient) = Self::collect_system_transfer_recipient_account(
//                             &transaction_accounts,
//                             &instruction.accounts,
//                         ) {
//                             Some((Self::get_channel_type(recipient), lamports))
//                         } else {
//                             None
//                         }
//                     }
//                     _ => None,
//                 },
//                 Err(e) => {
//                     error!("deserialize system instruction error: {e:?}");
//                     None
//                 }
//             }
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
