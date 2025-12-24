pub fn create_account_instruction_data(lamports: u64, space: u64, owner: &[u8; 32]) -> Vec<u8> {
    let mut data = Vec::with_capacity(52);
    data.extend_from_slice(&0u32.to_le_bytes()); // CreateAccount instruction discriminator
    data.extend_from_slice(&lamports.to_le_bytes());
    data.extend_from_slice(&space.to_le_bytes());
    data.extend_from_slice(owner);
    data
}