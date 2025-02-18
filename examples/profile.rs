use special_numbers::{Lover, string_to_u8, u8_to_string};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::signer::keypair::read_keypair_file;
use solana_sdk::{
	commitment_config::CommitmentConfig,
	instruction::{AccountMeta, Instruction},
	pubkey::Pubkey,
	signature::{Keypair, Signer},
	system_program,
	transaction::Transaction,
};
use std::{env, fs, path::Path, 
str::FromStr, io::Write, result::Result};

pub struct Profile {
	program_id: Pubkey,
	client: RpcClient,
	payer: Keypair,
	lover_keypair: Keypair,
	pub first_run: bool,
}

impl Profile {
	pub fn new() -> Self {
		// Program ID
		let program_id = Pubkey::from_str("2p6Zufn2gGtqmcYLVn8PW9efJLhgupUbM5Add2yCfkQ1").unwrap();
		// Connect to the Solana devnet
		let rpc_url = String::from("http://127.0.0.1:8899");
		//let rpc_url = "https://api.devnet.solana.com";
		let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
		// Generate a new keypair for the payer
	 // Path to the Solana CLI keypair file (default location)
		let keypair_path = std::env::var("SOLANA_KEYPAIR_PATH")
				.unwrap_or_else(|_| "~/.config/solana/id.json".to_string());

		// Expand tilde (~) to the full home directory path
		let keypair_path = shellexpand::tilde(&keypair_path).to_string();

		// Read the keypair JSON file
		let keypair_data = fs::read_to_string(&keypair_path).expect("Failed to read keypair file");

    // Deserialize into a Keypair
    let payer = Keypair::from_bytes(
        &serde_json::from_str::<Vec<u8>>(&keypair_data).expect("Invalid keypair file"),
    )
    .expect("Failed to parse keypair");

	
		let user_data = get_account_keypair();
		let lover_keypair = user_data.0;
		let first_run = user_data.1;
		Self { program_id, client, payer, lover_keypair, first_run }
	}

	pub fn get_chain_account(&mut self) -> Result<Lover, String> {
		match self.client.get_account(&self.lover_keypair.pubkey()) {
			Ok(account) => {
				match Lover::try_from_slice(&account.data) {
					Ok(lover) => {
						return Ok(lover);
					},
					Err(e) => {
						return Err(format!("failed t odeserialize account: {}", e));
					}
				}
			}
			Err(_) => {
				self.first_run = true;
				return Err("couldnt get".to_string());
			}
		}
	}

	pub fn set_name(&self, new_name: String) {
		let name = string_to_u8(&new_name);
		let mut lover_data = vec![0];
		lover_data.extend_from_slice(&name);//vec![];
		let instruction = solana_program::instruction::Instruction::new_with_bytes(
					self.program_id, 
					&lover_data, 
					vec![
						AccountMeta::new(self.lover_keypair.pubkey(), true),
						AccountMeta::new(self.payer.pubkey(), true),
						AccountMeta::new_readonly(system_program::id(), false),
					],
			);

		// Add the instruction to a new transaction
		let mut transaction = 
			Transaction::new_with_payer(&[instruction], Some(&self.payer.pubkey()));
		transaction.sign(&[&self.payer, &self.lover_keypair], self.client.get_latest_blockhash().unwrap());
		// Send and confirm the transaction
		match self.client.send_and_confirm_transaction(&transaction) {
				Ok(signature) => {
					println!("Transaction Signature: {}", signature);
					println!("What a beautiful name its great to meet you {}", new_name);
				},
				Err(err) => eprintln!("Error sending transaction: {}", err),
		}
	}

	pub fn set_special_number(&self, number: u64) -> bool {
		let mut number_data = vec![1];
		number_data.extend_from_slice(&number.to_le_bytes());
		let number_instruction = 
			solana_program::instruction::Instruction::new_with_bytes(
				self.program_id,
				&number_data,
				vec![AccountMeta::new(self.lover_keypair.pubkey(), true)]
			);
		let mut transaction = 
			Transaction::new_with_payer(&[number_instruction], Some(&self.payer.pubkey()));
		transaction.sign(&[&self.payer, &self.lover_keypair], self.client.get_latest_blockhash().unwrap());
		match self.client.send_and_confirm_transaction(&transaction) {
				Ok(signature) => {
					println!("Transaction Signature: {}", signature);
					println!("Wow thats so you! I can really see you have a connection with {}", number);
					
				},
				Err(err) => eprintln!("Error sending transaction: {}", err),
		}
		true
	}
	pub fn is_special_number_taken(&self, number: u64) -> bool {
		let mut taken = false;	
		// Fetch all accounts owned by the program
		match self.client.get_program_accounts(&self.program_id) {
				Ok(accounts) => {
						for (pubkey, account) in accounts {
							println!("Account Pubkey: {}", pubkey);
							println!("Account Data: {:?}", account.data); // Account data in bytes
							println!("Account Lamports: {}", account.lamports);
							println!("---");
							match Lover::try_from_slice(&account.data) {
								Ok(lover) => {
									if lover.special_numbers[0] == number {
										println!("you cant have that number, its connection is with {}", u8_to_string(lover.name));
									}
									taken = true;
								},
								Err(e) => eprintln!("Failed to deserialize: {}", e),
							}
						}

				}
				Err(err) => {
						eprintln!("Failed to fetch program accounts: {}", err);
				}
		}
		taken
	}
}

fn get_account_keypair() -> (Keypair, bool) {
	let binding = env::current_dir().unwrap();
	let k_path_str = binding.to_str().expect("Failed to convert current directry to a pth string");
	let keypair_path = format!("{}{}", k_path_str, "/account-keypair.json");
	let file_check = Path::new(&keypair_path);
	let mut new_user = false;
	if file_check.exists() {
		println!("this path exists");
	} else {
		println!("this file doesn't exist");
		let keypair = Keypair::new();
		// Get the secret key as a byte array
		let secret_key = keypair.to_bytes();
		// Convert the secret key to a JSON-friendly format (Vec<u8>)
		let secret_key_vec: Vec<u8> = secret_key.to_vec();
		// Specify the output file path
		let file_path = "account-keypair.json";
		// Write the secret key to the JSON file
		let mut file = fs::File::create(file_path).expect("File couldn't be created");
		let _ = file.write_all(serde_json::to_string(&secret_key_vec).unwrap().as_bytes());
		new_user = true;
	}

  let account_keypair = read_keypair_file(Path::new(&keypair_path)).expect("Failed to load keypair");
   println!("Loaded keypair: {:?}", account_keypair.pubkey());
	(account_keypair, new_user)
}

fn main() {
	todo!();
}

