use special_numbers::{Lover, NAME,MAX_SPECIAL, u8_to_string};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use borsh::BorshDeserialize;
use std::{collections::HashMap, thread, time::Duration};

// offline ledger, storing other Lover accounts on the blockchain
pub struct Ledger {
	pages: HashMap<String, [u64; MAX_SPECIAL]>,
	count: u64,
}

impl Ledger {
	pub fn new() -> Self {
		Self { pages: HashMap::new(), count: 0}
	}

	pub fn update(&mut self, client: &RpcClient, program_id: &Pubkey) -> Result<(),String> {
		self.pages = HashMap::new();
		match client.get_program_accounts(program_id) {
			Ok(accounts) => {
				for (_, account) in accounts {
					match Lover::try_from_slice(&account.data) {
						Ok(lover) => {
							self.add(lover.name, lover.special_numbers);
						}
						Err(_) => {}
					}
				}
			}
			Err(err) => {
				eprintln!("Failed to fetch program accounts: {}", err);
				return Err("couldn't get accounts from blockchain".to_string());
			}
		}
		Ok(())
	}
	
	pub fn add(&mut self, name: [u8; NAME], numbers: [u64; MAX_SPECIAL]) {
		self.pages.insert(u8_to_string(name), numbers);
		for i in 0..MAX_SPECIAL {
			if numbers[i] != 0 {
				self.count += 1;
			}
		}
	}

	pub fn read(&self) {
		let interval = 30;
		for _i in 0..40 {
			println!("");
			thread::sleep(Duration::from_millis(interval));
		}
		
		for (_name, numbers) in &self.pages {
			for i in 0..MAX_SPECIAL {
				if numbers[i] != 0 {
					println!("{}", numbers[i]);
					thread::sleep(Duration::from_millis(interval));
				}
			}
		}
		for _i in 0..40 {
			println!("");
			thread::sleep(Duration::from_millis(interval));
		}
	}

	pub fn check_special_number(&self, number: u64) -> Option<String> {
		for (name, numbers) in &self.pages {
			for i in 0..MAX_SPECIAL {
				if numbers[i] == number {
					return Some(name.to_string());
				} else if numbers[i] == 0 {
					break;
				}
			}
		}
		None
	}
}
