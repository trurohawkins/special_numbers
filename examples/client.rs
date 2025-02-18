pub mod profile;

use special_numbers::{string_to_u8, u8_to_string, Lover, };
use crate::profile::Profile;
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
	let mut poppy: Profile = Profile::new();
	let mut current_lover = Lover {
		name: string_to_u8(""),
		special_number: 0,
	};
	match poppy.get_chain_account() {
		Ok(lover) => {
			if lover.special_number != 0 {
				println!("Welcome back {}! You have a special connection with {}"
					, u8_to_string(lover.name), lover.special_number);
					current_lover.special_number = lover.special_number;
			} else {
				println!("Welcome back {}! You still havent found a connection have you?"
					, u8_to_string(lover.name));
			}
			current_lover.name = lover.name;
		}
		Err(_) => {}
	}
	/*
	// Get account information
	match client.get_account(&lover_keypair.pubkey()) {
			Ok(account) => {
				match Lover::try_from_slice(&account.data) {
					Ok(lover) => {
					},
					Err(e) => eprintln!("Failed to deserialize: {}", e),
				}
			}
			Err(_) => {first_run = true;}//println!("Failed to fetch account info: {}", err),
	}
	*/
	let mut input = String::new();
	if poppy.first_run {
		println!("Welcome! what is your name?");
		stdin().read_line(&mut input).expect("Failed to read");
		input.pop();
		current_lover.name = string_to_u8(&input);
		poppy.set_name(input);
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
					if poppy.set_special_number(new_number) {
						current_lover.special_number = new_number;
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

