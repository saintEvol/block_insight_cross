use solana_pubkey::Pubkey;
use tracing::error;

///   * Single owner/delegate
///   0. `[writable]` The source account.
///   1. `[]` The token mint.
///   2. `[writable]` The destination account.
///   3. `[signer]` The source account's owner/delegate.
///
pub struct TransferCheckedAccounts {
    pub source: Pubkey,
    pub mint: Pubkey,
    pub destination: Pubkey,
    pub authority: Pubkey,
}

pub struct TransferCheckedInstruction {
    pub accounts: TransferCheckedAccounts,
    pub amount: u64,
    pub decimals: u8,
}

// pub fn try_parse_transfer_checked_instruction(
//     transaction_accounts: &TransactionAccounts,
//     instruction: &InnerInstruction,
// ) -> anyhow::Result<Option<TransferCheckedInstruction>> {
//     let program =
//         if let Some(program) = transaction_accounts.get(instruction.program_id_index as usize) {
//             program
//         } else {
//             return Ok(None);
//         };
//     // 检查程序id
//     if program != spl_token::ID.as_ref() {
//         return Ok(None);
//     }
//
//     let source = if let Some(source) = transaction_accounts.get(instruction.accounts[0] as usize) {
//         Pubkey::try_from_slice(source)?
//     } else {
//         return Ok(None);
//     };
//     let mint = if let Some(mint) = transaction_accounts.get(instruction.accounts[1] as usize) {
//         Pubkey::try_from_slice(mint)?
//     } else {
//         return Ok(None);
//     };
//     let destination =
//         if let Some(destination) = transaction_accounts.get(instruction.accounts[2] as usize) {
//             Pubkey::try_from_slice(destination)?
//         } else {
//             return Ok(None);
//         };
//     let authority =
//         if let Some(authority) = transaction_accounts.get(instruction.accounts[3] as usize) {
//             Pubkey::try_from_slice(authority)?
//         } else {
//             return Ok(None);
//         };
//     let accounts = TransferCheckedAccounts {
//         source,
//         mint,
//         destination,
//         authority,
//     };
//     let token_instruction = spl_token::instruction::TokenInstruction::unpack(&instruction.data)?;
//     match token_instruction {
//         TokenInstruction::TransferChecked { amount, decimals } => {
//             let rest = TransferCheckedInstruction {
//                 accounts,
//                 amount,
//                 decimals,
//             };
//
//             Ok(Some(rest))
//         }
//         _ => Ok(None),
//     }
// }

pub enum TransactionAccounts<'a, AccountType> {
    Static(Option<&'a [AccountType]>),
    Dynamic(DynamicTransactionAccounts<'a, AccountType>),
}

impl<'a, AccountType> TransactionAccounts<'a, AccountType> {
    pub fn contains(&self, account: &'a AccountType) -> bool
    where
        AccountType: PartialEq,
    {
        match &self {
            TransactionAccounts::Static(keys) => keys
                .map(|a| Self::slice_contains(a, account))
                .unwrap_or(false),
            TransactionAccounts::Dynamic(DynamicTransactionAccounts {
                account_keys,
                loaded_writable_accounts,
                loaded_readonly_accounts,
            }) => {
                if let Some(accounts) = account_keys {
                    if Self::slice_contains(accounts, account) {
                        return true;
                    }
                }

                if let Some(loaded_writable_accounts) = loaded_writable_accounts {
                    if Self::slice_contains(loaded_writable_accounts, account) {
                        return true;
                    }
                }

                if let Some(loaded_readonly_accounts) = loaded_readonly_accounts {
                    if Self::slice_contains(loaded_readonly_accounts, account) {
                        return true;
                    }
                }

                false
            }
        }
    }

    pub fn from_accounts(
        account_keys: Option<&'a [AccountType]>,
        loaded_writable_accounts: Option<&'a [AccountType]>,
        loaded_readonly_accounts: Option<&'a [AccountType]>,
    ) -> Self {
        if loaded_readonly_accounts
            .map(|e| e.is_empty())
            .unwrap_or(true)
            && loaded_writable_accounts
                .map(|e| e.is_empty())
                .unwrap_or(true)
        {
            Self::Static(account_keys)
        } else {
            Self::Dynamic(DynamicTransactionAccounts {
                account_keys,
                loaded_writable_accounts,
                loaded_readonly_accounts,
            })
        }
    }

    pub fn all_accounts(&self) -> Vec<&AccountType> {
        match self {
            TransactionAccounts::Static(s) => {
                if let Some(account_keys) = s {
                    let mut ret = Vec::with_capacity(account_keys.len());
                    for a in *account_keys {
                        ret.push(a);
                    }
                    ret
                } else {
                    vec![]
                }
            }
            TransactionAccounts::Dynamic(d) => {
                let DynamicTransactionAccounts {
                    account_keys,
                    loaded_writable_accounts,
                    loaded_readonly_accounts,
                } = &d;
                let mut ret = Vec::with_capacity(self.accounts_num());
                if let Some(account_keys) = account_keys {
                    for a in *account_keys {
                        ret.push(a);
                    }
                }
                if let Some(writes) = loaded_writable_accounts {
                    for w in *writes {
                        ret.push(w);
                    }
                }
                if let Some(readonly) = loaded_readonly_accounts {
                    for r in *readonly {
                        ret.push(r);
                    }
                }

                ret
            }
        }
    }

    pub fn accounts_num(&self) -> usize {
        match &self {
            TransactionAccounts::Static(s) => s.map(|s| s.len()).unwrap_or(0),
            TransactionAccounts::Dynamic(d) => {
                let DynamicTransactionAccounts {
                    account_keys,
                    loaded_writable_accounts,
                    loaded_readonly_accounts,
                } = &d;
                let accounts_len = account_keys.map(|k| k.len()).unwrap_or(0);
                let write_len = loaded_writable_accounts.map(|k| k.len()).unwrap_or(0);
                let read_len = loaded_readonly_accounts.map(|k| k.len()).unwrap_or(0);

                accounts_len + write_len + read_len
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&'a AccountType> {
        match self {
            TransactionAccounts::Static(s) => s.map(|s| s.get(index)).flatten(),
            TransactionAccounts::Dynamic(d) => {
                let account_keys_len = d.account_keys.map(|a| a.len()).unwrap_or(0);
                // 如果索引在静态帐户区间内，直接使用此区间的帐户
                if index < account_keys_len {
                    return d.account_keys.map(|a| a.get(index)).flatten();
                }

                // 计算下一个区间还有多少
                // 0 0 => 0 (下一个区间0索引)
                // 0 1 => -1 (不用)
                // 1 1 => 0 下一个区间0索引位置
                // 10 2 => 8 下一个区间索引8位置
                let next_index = index as i32 - account_keys_len as i32;
                let writable = d.loaded_writable_accounts;
                if next_index < 0 {
                    error!("next index < 0");
                    return None;
                }
                if (next_index as usize) < writable.map(|w| w.len()).unwrap_or(0) {
                    return writable
                        .map(|accounts| accounts.get(next_index as usize))
                        .flatten();
                }

                let next_index = next_index as i32 - writable.map(|w| w.len()).unwrap_or(0) as i32;
                if next_index < 0 {
                    error!("next index < 0");
                    return None;
                }
                d.loaded_readonly_accounts
                    .map(|accounts| accounts.get(next_index as usize))
                    .flatten()
            }
        }
    }

    fn slice_contains(slice: &[AccountType], account: &AccountType) -> bool
    where
        AccountType: PartialEq,
    {
        for a in slice {
            if a == account {
                return true;
            }
        }

        false
    }
}

pub struct DynamicTransactionAccounts<'a, AccountType> {
    account_keys: Option<&'a [AccountType]>,
    loaded_writable_accounts: Option<&'a [AccountType]>,
    loaded_readonly_accounts: Option<&'a [AccountType]>,
}
