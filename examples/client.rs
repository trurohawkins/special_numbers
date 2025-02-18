pub mod profile;

use special_numbers::{string_to_u8, u8_to_string, Lover, };
use crate::profile::Profile;
use std::{
		io::stdin,
		thread, sync::{Arc, Mutex, mpsc},
};

#[tokio::main]
async fn main() {
	let mut poppy: Profile = Profile::new().expect("no solana-cli keypair");
	let mut current_lover = Lover {
				name: string_to_u8(""),
				special_numbers: [0,0,0,0,0],
				love: 1,
	};
	println!("Hey who is there?\nMy eyes aren't what they used to be...");
	let mut input = String::new();
	stdin().read_line(&mut input).expect("Failed to read");
	input.pop();
	current_lover.name = string_to_u8(&input);
	poppy.set_name(input);
	if !poppy.first_run {
		println!("Welcome back {}!", u8_to_string(current_lover.name));
		match poppy.get_chain_account() {
			Ok(lover) => {
				if lover.special_numbers[0] != 0 {
					println!("I see here you have a special connection with {}", lover.special_numbers[0]);

					current_lover.special_numbers = lover.special_numbers;
				} else {
					println!("You still havent found a connection have you?");
				}
			}
			Err(e) => {println!("couldnt get account: {:?}", e);}
		}
	} else {
		println!("Welcome, I hope you have a good time here");
	}
	loop {
		if current_lover.special_numbers[0] == 0 {
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
						current_lover.special_numbers[0] = new_number;
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

