use special_numbers::{Lover, string_to_u8, u8_to_string ,MAX_SPECIAL};
use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::keypair::read_keypair_file;
use solana_sdk::{
		commitment_config::CommitmentConfig,
		instruction::AccountMeta,
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
							lover: Option<Keypair>,
							pub first_run: bool,
}

impl Profile {
	pub fn new() -> Result<Self, String> {
		let program_id = Pubkey::from_str("2p6Zufn2gGtqmcYLVn8PW9efJLhgupUbM5Add2yCfkQ1").unwrap();
		let rpc_url = String::from("http://127.0.0.1:8899");
		//let rpc_url = "https://api.devnet.solana.com";
		let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
		
		// use solana client keypir for payment
		let keypair_path = std::env::var("SOLANA_KEYPAIR_PATH")
			.unwrap_or_else(|_| "~/.config/solana/id.json".to_string());
		let keypair_path = shellexpand::tilde(&keypair_path).to_string();
		match fs::read_to_string(&keypair_path) {
			Ok(keypair_data) => {
				let payer = Keypair::from_bytes(
						&serde_json::from_str::<Vec<u8>>(&keypair_data).expect("Invalid keypair file"),
						)
					.expect("Failed to parse keypair");
				Ok(Self { program_id, client, payer, lover: None, first_run: false })
			}
			Err(e) => {
				println!("couldn't read your keypair for the solana cli, is installed?\n
					could it have been moved? {:?}", e);
				Err("no solana kaypair found".to_string())
			}
		}

	}

	pub fn get_chain_account(&mut self) -> Result<Lover, String> {
		match &self.lover {
			Some(lover_keypair) => {
				match self.client.get_account(&lover_keypair.pubkey()) {
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
			None => {return Err("no lover loaded".to_string());}
		}
	}

	pub fn set_name(&mut self, new_name: String) {
		match &self.lover {
			Some(_lover_keypair) => {
				println!("You already have a name");
			}
			None => {
				// returns key pair, and bool idnicating if it was created or not
				let user_data = get_account_keypair(&new_name);
				let lover_keypair = user_data.0;
				self.first_run = user_data.1;
				// if we already had a keypair file, we assume that we have a on chain account as well
				if self.first_run {
					let name = string_to_u8(&new_name);
					let mut lover_data = vec![0];
					lover_data.extend_from_slice(&name);
					let instruction = solana_program::instruction::Instruction::new_with_bytes(
							self.program_id, 
							&lover_data, 
							vec![
							AccountMeta::new(lover_keypair.pubkey(), true),
							AccountMeta::new(self.payer.pubkey(), true),
							AccountMeta::new_readonly(system_program::id(), false),
							],
					);

					let mut transaction = 
						Transaction::new_with_payer(&[instruction], Some(&self.payer.pubkey()));
					transaction.sign(&[&self.payer, &lover_keypair], self.client.get_latest_blockhash().unwrap());
					match self.client.send_and_confirm_transaction(&transaction) {
						Ok(_signature) => {},
							Err(err) => eprintln!("Error sending transaction: {}", err),
					}
				}
				self.lover = Some(lover_keypair);
			}
		}
	}

	pub fn set_special_number(&self, number: u64) -> bool {
		match &self.lover {
			Some(lover_keypair) => {
				if !self.is_special_number_taken(number) {
					let mut number_data = vec![1];
					number_data.extend_from_slice(&number.to_le_bytes());
					let number_instruction = 
						solana_program::instruction::Instruction::new_with_bytes(
								self.program_id,
								&number_data,
								vec![AccountMeta::new(lover_keypair.pubkey(), true)]
								);
					let mut transaction = 
						Transaction::new_with_payer(&[number_instruction], Some(&self.payer.pubkey()));
					transaction.sign(&[&self.payer, &lover_keypair], self.client.get_latest_blockhash().unwrap());
					match self.client.send_and_confirm_transaction(&transaction) {
						Ok(_signature) => {
							println!("Wow thats so you! I can really see you have a connection with {}", number);

						},
							Err(err) => eprintln!("Error sending transaction: {}", err),
					}
					return true;
				}
			}
			None => {println!("Who are you to have a connection?");}
		}
		false
	}

	pub fn is_special_number_taken(&self, number: u64) -> bool {
		let mut taken = false;	
		// Fetch all accounts owned by the program
		match self.client.get_program_accounts(&self.program_id) {
			Ok(accounts) => {
				for (_, account) in accounts {
					match Lover::try_from_slice(&account.data) {
						Ok(lover) => {
							for i in 0..MAX_SPECIAL {
								if lover.special_numbers[i] == number {
									println!("you cant have that number, its connection is with {}", u8_to_string(lover.name));
									taken = true;
								} else if lover.special_numbers[i] == 0 {
									break;
								}
							}
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

fn get_account_keypair(name: &str) -> (Keypair, bool) {
	let binding = env::current_dir().unwrap();
	let k_path_str = binding.to_str().expect("Failed to convert current directry to a pth string");
	let dir_path = "lovers/";
	let filename = format!("{}{}{}", dir_path, name, "-keypair.json"); 
	let keypair_path = format!("{}{}{}", k_path_str, "/", filename);
	let file_check = Path::new(&keypair_path);
	let mut new_user = false;
	if !file_check.exists() {
		// check for directory
		let dir_path = Path::new(dir_path);
		if !dir_path.exists() {
			let _ = fs::create_dir_all(dir_path);
		}
		let keypair = Keypair::new();
		let secret_key = keypair.to_bytes();
		// Convert the secret key to a JSON-friendly format (Vec<u8>)
		let secret_key_vec: Vec<u8> = secret_key.to_vec();
		// Write the secret key to the JSON file
		let mut file = fs::File::create(filename).expect("File couldn't be created");
		let _ = file.write_all(serde_json::to_string(&secret_key_vec).unwrap().as_bytes());
		new_user = true;
	}

	let account_keypair = read_keypair_file(Path::new(&keypair_path)).expect("Failed to load keypair");
	(account_keypair, new_user)
}

// vestigial, needed so that cargo build-sbf doesnt complain
fn main() {
	todo!();
}

