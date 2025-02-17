use special_numbers::{string_to_u8, u8_to_string, Lover};
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
//use solana_cli_config::Config;
use std::{
	str::FromStr,
	path::Path,
	env,
	fs::File,
	io::{Write, stdin},
	thread, sync::{Arc, Mutex, mpsc},
};
use serde_json;

#[tokio::main]
async fn main() {
	// Program ID
	let program_id = Pubkey::from_str("4eHaDppPZ6rwiNKfhEkQNMVn4wTijit9J2KbPSBi4saa").unwrap();
	// Connect to the Solana devnet
	let rpc_url = String::from("http://127.0.0.1:8899");
	let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
	// Generate a new keypair for the payer
	let payer = Keypair::new();
	println!("Using keypair: {:?}", payer.pubkey());
	// Request airdrop
	let airdrop_amount = 5_000_000_000; // 1 SOL
	let signature = client
		.request_airdrop(&payer.pubkey(), airdrop_amount)
		.expect("Failed to request airdrop");
	
	// Wait for airdrop confirmation
	loop {
		let confirmed = client.confirm_transaction(&signature).unwrap();
		if confirmed {
			break;
		}
	}
	let user_data = get_account_keypair();
	let lover_keypair = user_data.0;
	let mut fresh_user = user_data.1;
	let mut current_lover = Lover {
		name: string_to_u8(""),
		special_number: 0,
	};
	// Get account information
	match client.get_account(&lover_keypair.pubkey()) {
			Ok(account) => {
				match Lover::try_from_slice(&account.data) {
					Ok(lover) => {
							if lover.special_number != 0 {
								println!("Welcome back {}You have a special connection with {}"
									, u8_to_string(lover.name), lover.special_number);
									current_lover.special_number = lover.special_number;
							} else {
								println!("Welcome back {}You still havent found a connection have you?"
									, u8_to_string(lover.name));
							}
							current_lover.name = lover.name;
					},
					Err(e) => eprintln!("Failed to deserialize: {}", e),
				}
			}
			Err(_) => {fresh_user = true;}//println!("Failed to fetch account info: {}", err),
	}
	let mut input = String::new();
	if fresh_user {
		println!("Welcome! what is your name?");
		stdin().read_line(&mut input).expect("Failed to read");
		input.pop();
		let name = string_to_u8(&input);
		let mut lover_data = vec![0];
		lover_data.extend_from_slice(&name);//vec![];
		let instruction = solana_program::instruction::Instruction::new_with_bytes(
					program_id, 
					&lover_data, 
					vec![
						AccountMeta::new(lover_keypair.pubkey(), true),
						AccountMeta::new(payer.pubkey(), true),
						AccountMeta::new_readonly(system_program::id(), false),
					],
			);

		// Add the instruction to a new transaction
		let mut transaction = 
			Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
		transaction.sign(&[&payer, &lover_keypair], client.get_latest_blockhash().unwrap());
		// Send and confirm the transaction
		match client.send_and_confirm_transaction(&transaction) {
				Ok(signature) => {
					println!("Transaction Signature: {}", signature);
					println!("What a beautiful name its great to meet you {}", input);
					current_lover.name = name;
				},
				Err(err) => eprintln!("Error sending transaction: {}", err),
		}
	}
	loop {
		if current_lover.special_number == 0 {
			println!("what is your special number?\nYou know the one that really gets you going?\nYou can tell me!");
			//First access the input message and read it
			let mut input = String::new();
			stdin().read_line(&mut input).expect("Failed to read");
			let letter: u8 = input.chars().next().unwrap() as u8;
			//println!("{}", letter);
			if letter == 27 {
				println!("god bye I love you");
				break;
			} else {
				input.pop();
				if is_integer(&input) {
					println!("thats a good number");
					let new_number = input.parse::<u64>().unwrap();
					let mut number_data = vec![1];
					number_data.extend_from_slice(&new_number.to_le_bytes());
					let number_instruction = 
						solana_program::instruction::Instruction::new_with_bytes(
							program_id,
							&number_data,
							vec![AccountMeta::new(lover_keypair.pubkey(), true)]
						);
						let mut transaction = Transaction::new_with_payer(&[number_instruction], Some(&payer.pubkey()));
						transaction.sign(&[&payer, &lover_keypair], client.get_latest_blockhash().unwrap());
						match client.send_and_confirm_transaction(&transaction) {
								Ok(signature) => {
									println!("Transaction Signature: {}", signature);
									println!("Wow thats so you! I can really see you have a connection with {}", input);
									current_lover.special_number = new_number;
								},
								Err(err) => eprintln!("Error sending transaction: {}", err),
						}

				} else {
					if letter == b'-' {
						println!("we don't mess around with that, perhaps you should go");
						break;
					}
					println!("{} isnt a number silly!", input);
				}
			}
		} else {
			break;
		}
	}
	
fn is_integer(s: &str) -> bool {
    s.parse::<u64>().is_ok()
}

// gotta test
fn is_special_number_taken(number: u64, client: RpcClient, program_id: Pubkey) -> bool {
	let mut taken = false;	
	// Fetch all accounts owned by the program
	match client.get_program_accounts(&program_id) {
			Ok(accounts) => {
					for (pubkey, account) in accounts {
						println!("Account Pubkey: {}", pubkey);
						println!("Account Data: {:?}", account.data); // Account data in bytes
						println!("Account Lamports: {}", account.lamports);
						println!("---");
						match Lover::try_from_slice(&account.data) {
							Ok(lover) => {
								if lover.special_number == number {
									println!("you cant have that number")
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
	/*
	
	let _ = get_account_keypair();
	let mut new_user = false;


 // Create the instruction
	let instruction = Instruction::new_with_borsh(
			program_id,
			&(),    // Empty instruction data
			vec![], // No accounts needed
	);

	// Add the instruction to new transaction
	let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
	transaction.sign(&[&payer], client.get_latest_blockhash().unwrap());

	// Send and confirm the transaction
	match client.send_and_confirm_transaction(&transaction) {
			Ok(signature) => println!("Transaction Signature: {}", signature),
			Err(err) => eprintln!("Error sending transaction: {}", err),
	}
	*/
}
/*
pub fn input(tx: mpsc::Sender<String>) {
	let mut first_inp = false;//true;
	loop {
		println!("lets go");
		if first_inp {
			match tx.send("C".to_string()) {
				Ok(_) => {},
				Err(e) => {println!("input broadcast to main thread failed {}", e);}
			}
			first_inp = false;
		} else {
			//Allow sender t oenter message input
			let mut input = String::new();
			//First access the input message and read it
			stdin().read_line(&mut input).expect("Failed to read");
			match tx.send(input) {
				Ok(_) => {},
				Err(e) => {println!("input broadcast to main thread failed {}", e);}
			}
		}
	}
}
*/
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
		let mut file = File::create(file_path).expect("File couldn't be created");
		let _ = file.write_all(serde_json::to_string(&secret_key_vec).unwrap().as_bytes());
		new_user = true;
	}

  let account_keypair = read_keypair_file(Path::new(&keypair_path)).expect("Failed to load keypair");
   println!("Loaded keypair: {:?}", account_keypair.pubkey());
	(account_keypair, new_user)
}

