use anchor_lang::prelude::*;

pub fn is_discriminator_already_set<'info>(account: &UncheckedAccount<'info>) -> Result<bool> {
    let data = account.try_borrow_data()?;
    let mut disc_bytes = [0u8; 8];
    disc_bytes.copy_from_slice(&data[..8]);
    let discriminator = u64::from_le_bytes(disc_bytes);
    Ok(discriminator != 0)
}

pub fn try_deserialize_unchecked<'info, T: AccountDeserialize>(
    account: &UncheckedAccount<'info>,
) -> Result<T> {
    let account_data = account.try_borrow_data()?;
    let mut account_data_slice: &[u8] = &account_data;
    T::try_deserialize_unchecked(&mut account_data_slice)
}
