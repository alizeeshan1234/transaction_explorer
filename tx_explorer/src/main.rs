use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::signature::Signature;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;
use std::error::Error;

#[derive(Debug)]
struct TransactionInfo {
    signature: String,
    slot: u64,
    block_time: Option<i64>,
    success: bool,
    err: Option<String>,
    fee: u64,
    account_keys: Vec<String>,
    instructions_count: usize,
}

#[derive(Debug)]
struct AddressInfo {
    address: String,
    lamports: u64,
    owner: String,
    executable: bool,
    rent_epoch: u64,
}

struct SolanaExplorer {
    client: RpcClient,
}

impl SolanaExplorer {
    fn new(rpc_url: &str) -> Self {
        let client = RpcClient::new(rpc_url.to_string());
        SolanaExplorer { client }
    }

    fn get_transaction(&self, signature: &str) -> Result<TransactionInfo, Box<dyn Error>> {
        let sig = Signature::from_str(signature)?;
        let tx = self.client.get_transaction_with_config(
            &sig,
            solana_client::rpc_config::RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: None,
                max_supported_transaction_version: Some(0),
            },
        )?;

        let success = tx.transaction.meta.as_ref().map(|m| m.err.is_none()).unwrap_or(false);
        let err = tx.transaction.meta.as_ref().and_then(|m| m.err.as_ref()).map(|e| format!("{:?}", e));
        let fee = tx.transaction.meta.as_ref().map(|m| m.fee).unwrap_or(0);

        let (account_keys, instructions_count) = match &tx.transaction.transaction {
            solana_transaction_status::EncodedTransaction::Json(encoded_tx) => {
                match &encoded_tx.message {
                    solana_transaction_status::UiMessage::Raw(raw_msg) => {
                        let keys: Vec<String> = raw_msg.account_keys.clone();
                        let instr_count = raw_msg.instructions.len();
                        (keys, instr_count)
                    }
                    solana_transaction_status::UiMessage::Parsed(parsed_msg) => {
                        let keys: Vec<String> = parsed_msg
                            .account_keys
                            .iter()
                            .map(|acc| acc.pubkey.clone())
                            .collect();
                        let instr_count = parsed_msg.instructions.len();
                        (keys, instr_count)
                    }
                }
            }
            _ => (vec![], 0),
        };

        Ok(TransactionInfo {
            signature: signature.to_string(),
            slot: tx.slot,
            block_time: tx.block_time,
            success,
            err,
            fee,
            account_keys,
            instructions_count,
        })
    }

    fn get_account(&self, address: &str) -> Result<AddressInfo, Box<dyn Error>> {
        let pubkey = Pubkey::from_str(address)?;
        let account = self.client.get_account(&pubkey)?;

        Ok(AddressInfo {
            address: address.to_string(),
            lamports: account.lamports,
            owner: account.owner.to_string(),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        })
    }

    fn get_address_signatures(&self, address: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let pubkey = Pubkey::from_str(address)?;
        let signatures = self.client.get_signatures_for_address_with_config(
            &pubkey,
            GetConfirmedSignaturesForAddress2Config {
                before: None,
                until: None,
                limit: Some(limit),
                commitment: None,
            },
        )?;

        Ok(signatures.iter().map(|s| s.signature.clone()).collect())
    }

    fn get_cluster_info(&self) -> Result<ClusterInfo, Box<dyn Error>> {
        let version = self.client.get_version()?;
        let slot = self.client.get_slot()?;

        Ok(ClusterInfo {
            solana_version: version.solana_core,
            current_slot: slot,
            network_active: true,
        })
    }
}

#[derive(Debug)]
struct ClusterInfo {
    solana_version: String,
    current_slot: u64,
    network_active: bool,
}

fn print_transaction_info(tx: &TransactionInfo) {
    println!("\n═══════════════════════════════════════");
    println!("TRANSACTION DETAILS");
    println!("═══════════════════════════════════════");
    println!("Signature:        {}", tx.signature);
    println!("Slot:             {}", tx.slot);
    println!("Block Time:       {:?}", tx.block_time);
    println!("Status:           {}", if tx.success { "✓ Success" } else { "✗ Failed" });
    if let Some(err) = &tx.err {
        println!("Error:            {}", err);
    }
    println!("Fee (lamports):    {}", tx.fee);
    println!("Instructions:     {}", tx.instructions_count);
    println!("Accounts ({}):", tx.account_keys.len());
    for (i, key) in tx.account_keys.iter().enumerate() {
        println!("  {}. {}", i + 1, key);
    }
    println!("═══════════════════════════════════════\n");
}

fn print_address_info(addr: &AddressInfo) {
    println!("\n═══════════════════════════════════════");
    println!("ACCOUNT DETAILS");
    println!("═══════════════════════════════════════");
    println!("Address:     {}", addr.address);
    println!("Balance:     {} SOL", addr.lamports as f64 / 1_000_000_000.0);
    println!("Lamports:    {}", addr.lamports);
    println!("Owner:       {}", addr.owner);
    println!("Executable:  {}", addr.executable);
    println!("Rent Epoch:  {}", addr.rent_epoch);
    println!("═══════════════════════════════════════\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let explorer = SolanaExplorer::new("https://api.mainnet-beta.solana.com");

    match explorer.get_cluster_info() {
        Ok(info) => {
            println!("\nCluster Version: {}", info.solana_version);
            println!("Current Slot: {}", info.current_slot);
        }
        Err(e) => println!("Error fetching cluster info: {}", e),
    }

    let tx_signature = "your_transaction_signature_here";
    match explorer.get_transaction(tx_signature) {
        Ok(tx) => print_transaction_info(&tx),
        Err(e) => println!("Transaction not found: {}", e),
    }

    let address = "11111111111111111111111111111111";
    match explorer.get_account(address) {
        Ok(account) => print_address_info(&account),
        Err(e) => println!("Account not found: {}", e),
    }

    match explorer.get_address_signatures(address, 10) {
        Ok(sigs) => {
            println!("Recent signatures for {}:", address);
            for (i, sig) in sigs.iter().enumerate() {
                println!("  {}. {}", i + 1, sig);
            }
        }
        Err(e) => println!("Error fetching signatures: {}", e),
    }

    Ok(())
}