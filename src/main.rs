use web3::types::{BlockId, BlockNumber, U64, H160, H256, U256};
use web3::{Web3, Transport};
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

#[derive(Debug)]
pub struct BlockAnalysis {
    block_info: BlockInfo,
    state_changes: Vec<StateChange>,
}

#[derive(Debug)]
pub struct BlockInfo {
    block_number: u64,
    timestamp: u64,
    hash: String,
    parent_hash: String,
    nonce: Option<String>,
    miner: String,
    difficulty: String,
    total_difficulty: Option<String>,
    size: u64,
    gas_used: u64,
    gas_limit: u64,
    transactions: Vec<TransactionInfo>,
}

#[derive(Debug)]
pub struct TransactionInfo {
    hash: H256,
    from: H160,
    to: Option<H160>,
    value: U256,
    gas_used: Option<U256>,
}

#[derive(Debug)]
struct StateChange {
    address: H160,
    balance_change: Option<U256>,
    nonce_change: Option<U256>,
}

pub async fn analyze_block<T: Transport>(
    web3: &Web3<T>,
    block_number: Option<u64>
) -> Result<BlockAnalysis, Box<dyn Error>> {
    // Get block info
    let block_info = get_block_info(web3, block_number).await?;

    // Get state changes
    let state_changes = get_state_changes(web3, &block_info).await?;

    Ok(BlockAnalysis {
        block_info,
        state_changes,
    })
}

async fn get_block_info<T: Transport>(
    web3: &Web3<T>,
    block_number: Option<u64>
) -> Result<BlockInfo, Box<dyn Error>> {
    // Determine block number or use 'latest'
    let block_id = match block_number {
        Some(num) => BlockId::Number(BlockNumber::Number(U64::from(num))),
        None => BlockId::Number(BlockNumber::Latest),
    };

    // Fetch block with full transaction objects
    let block = web3.eth().block_with_txs(block_id).await?
        .ok_or("Block not found")?;

    // Get transaction receipts for gas used
    let mut transactions = Vec::new();
    for tx in block.transactions {
        let receipt = web3.eth().transaction_receipt(tx.hash).await?;

        transactions.push(TransactionInfo {
            hash: tx.hash,
            from: tx.from.ok_or("Transaction missing 'from' address")?,
            to: tx.to,
            value: tx.value,
            gas_used: receipt.and_then(|r| r.gas_used),
        });
    }

    // Create BlockInfo struct with fetched data
    let block_info = BlockInfo {
        block_number: block.number.unwrap().as_u64(),
        timestamp: block.timestamp.as_u64(),
        hash: block.hash
            .map(|h| format!("{:?}", h))
            .unwrap_or_default(),
        parent_hash: format!("{:?}", block.parent_hash),
        nonce: block.nonce.map(|n| format!("{:?}", n)),
        miner: format!("{:?}", block.author),
        difficulty: block.difficulty.to_string(),
        total_difficulty: block.total_difficulty.map(|td| td.to_string()),
        size: block.size.unwrap_or_default().as_u64(),
        gas_used: block.gas_used.as_u64(),
        gas_limit: block.gas_limit.as_u64(),
        transactions,
    };

    Ok(block_info)
}

async fn get_state_changes<T: Transport>(
    web3: &Web3<T>,
    block_info: &BlockInfo,
) -> Result<Vec<StateChange>, Box<dyn Error>> {
    let mut changes = Vec::new();
    let mut addresses = HashMap::new();

    // Collect all addresses involved in transactions
    for tx in &block_info.transactions {
        addresses.insert(tx.from, true);
        if let Some(to) = tx.to {
            addresses.insert(to, true);
        }
    }

    // Add miner address
    let miner_str = block_info.miner.trim_matches(|c| c == '"' || c == '0' || c == 'x');
    if let Ok(miner) = H160::from_str(miner_str) {
        addresses.insert(miner, true);
    }

    // Previous block number
    let prev_block = block_info.block_number.saturating_sub(1);

    // Get balances and nonces for all addresses at both blocks
    for address in addresses.keys() {
        // Get previous state
        let prev_balance = web3.eth().balance(*address, Some(BlockNumber::Number(U64::from(prev_block)))).await?;
        let prev_nonce = web3.eth().transaction_count(*address, Some(BlockNumber::Number(U64::from(prev_block)))).await?;

        // Get current state
        let current_balance = web3.eth().balance(*address, Some(BlockNumber::Number(U64::from(block_info.block_number)))).await?;
        let current_nonce = web3.eth().transaction_count(*address, Some(BlockNumber::Number(U64::from(block_info.block_number)))).await?;

        // Check if state changed
        if prev_balance != current_balance || prev_nonce != current_nonce {
            changes.push(StateChange {
                address: *address,
                balance_change: Some(current_balance.overflowing_sub(prev_balance).0),
                nonce_change: Some(current_nonce.overflowing_sub(prev_nonce).0),
            });
        }
    }

    Ok(changes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Replace with your Ethereum node URL
    let transport = web3::transports::Http::new(
         "https://rpc-bitcoin-rollup-3mdaxk3vmn.t.conduit.xyz"  // or your node URL
    )?;
    let web3 = Web3::new(transport);

    // Replace with the block number you want to analyze, or use None for latest
    let block_number = Some(7408000u64);

    match analyze_block(&web3, block_number).await {
        Ok(analysis) => {
            println!("\nBlock Information:");
            println!("Block Number: {}", analysis.block_info.block_number);
            println!("Timestamp: {}", analysis.block_info.timestamp);
            println!("Hash: {}", analysis.block_info.hash);
            println!("Parent Hash: {}", analysis.block_info.parent_hash);
            println!("Nonce: {:?}", analysis.block_info.nonce);
            println!("Miner: {}", analysis.block_info.miner);
            println!("Difficulty: {}", analysis.block_info.difficulty);
            println!("Total Difficulty: {:?}", analysis.block_info.total_difficulty);
            println!("Size: {}", analysis.block_info.size);
            println!("Gas Used: {}", analysis.block_info.gas_used);
            println!("Gas Limit: {}", analysis.block_info.gas_limit);

            println!("\nTransactions:");
            for tx in &analysis.block_info.transactions {
                println!("\n  Hash: {:?}", tx.hash);
                println!("  From: {:?}", tx.from);
                println!("  To: {:?}", tx.to);
                println!("  Value: {} wei", tx.value);
                println!("  Gas Used: {:?}", tx.gas_used);
            }

            println!("\nState Changes:");
            for change in analysis.state_changes {
                println!("\nAddress: {:?}", change.address);

                if let Some(balance_change) = change.balance_change {
                    println!("Balance Change: {} wei", balance_change);
                }

                if let Some(nonce_change) = change.nonce_change {
                    println!("Nonce Change: {}", nonce_change);
                }
            }
        },
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
