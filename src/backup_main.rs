use web3::types::{BlockId, BlockNumber, U64};
use web3::{Web3, Transport};
use std::error::Error;
use std::str::FromStr;

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
    transaction_count: usize,
}

pub async fn get_block_info<T: Transport>(
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
        transaction_count: block.transactions.len(),
    };

    Ok(block_info)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Replace with your Ethereum node URL
    let transport = web3::transports::Http::new(
        "https://eth.llamarpc.com"
    )?;
    let web3 = Web3::new(transport);

    // Get the latest block
    match get_block_info(&web3, None).await {
        Ok(block_info) => {
            println!("Block Information:");
            println!("Block Number: {}", block_info.block_number);
            println!("Timestamp: {}", block_info.timestamp);
            println!("Hash: {}", block_info.hash);
            println!("Parent Hash: {}", block_info.parent_hash);
            println!("Nonce: {:?}", block_info.nonce);
            println!("Miner: {}", block_info.miner);
            println!("Difficulty: {}", block_info.difficulty);
            println!("Total Difficulty: {:?}", block_info.total_difficulty);
            println!("Size: {}", block_info.size);
            println!("Gas Used: {}", block_info.gas_used);
            println!("Gas Limit: {}", block_info.gas_limit);
            println!("Transaction Count: {}", block_info.transaction_count);
        },
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
