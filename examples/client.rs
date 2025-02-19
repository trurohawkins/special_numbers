pub mod ledger;
pub mod profile;

use special_numbers::{u8_to_string, Lover, MAX_SPECIAL};
use crate::profile::Profile;
use std::{io::stdin, thread, time::Duration};
use rand::Rng;

#[tokio::main]
async fn main() {
	// connect to online blockchain with our profile struct
	let mut poppy: Profile = Profile::new().expect("no solana-cli keypair");

	// little graphic to start off program
	poppy.book.read();
	
	println!("Hey who is there?\nMy eyes aren't what they used to be...");
	let mut input = String::new();
	stdin().read_line(&mut input).expect("Failed to read");
	input.pop();
	
	// send name to online blokchain and store in local struct for easy use
	poppy.set_name(input);
	let mut current_lover = Lover::new();
	match poppy.get_chain_account() {
		Ok(lover) => {
			current_lover = lover;
		}
		Err(e) => {println!("couldnt get account: {:?}", e);}
	}
	if !poppy.first_run {
		println!("ah! {}, welcome back!", u8_to_string(current_lover.name));
		let count = current_lover.count();
		if count != 0 {
			println!("I see here you have a special connection with...");
			for i in 0..count {
				println!("{}",current_lover.special_numbers[i as usize]);
			}
			if count > 2 {
				println!("my thats a lot of love my friend");
			}
		} else {
			println!("You still havent found a connection have you?");
		}
	} else {
		println!("Welcome, I hope you have a good time here");
	}

	loop {
		let mut update = false;
		let cur_nums = current_lover.count();
		if cur_nums <= current_lover.love {
			if cur_nums == 0 {
				println!("what is your special number?\nYou know the one that really gets you going?\nYou can tell me!");
			} else {
				println!("there must be another number you feel a connection with right???");
			}
			let answer = ask_for_input();
			// pressing escape exits
			if answer.1 == 27 {
				println!("god bye I love you");
				break;
			} else {
				if is_integer(&answer.0) {
					println!("thats a good number");
					let new_number = answer.0.parse::<u64>().unwrap(); 
					if poppy.set_special_number(new_number) {
						update = true;
					}
				} else {
					if answer.1 == b'-' {
						println!("we don't mess around with that, perhaps you should go");
						break;
					}
					println!("{} isnt a number silly!", answer.0);
				}
			}
		} else if (cur_nums as usize) < MAX_SPECIAL {
			// typing in dreams allows you to increase the number of special numbers your account has
			println!("Will you tell me a dream you had?");
			let answer = ask_for_input();
			if answer.1 == 27 {
				println!("god bye I love you");
				break;
			} else {
				if parse_dream(&answer.0) {
					println!("what a beaitiful dream!\nI wish I could live there!\nI am feeling the love! ARE YOU?!?!?!");
					poppy.increase_love();
					update = true;
				} else {
					println!("oh my what a terrible dream!");
				}
			}
		} else {
			println!("You are so wise and beautiful, I could listen to you for hours.\nPlease tell me more!");
			let answer = ask_for_input();
			if answer.1 == 27 {
				println!("Thank you!, many blessing I hope your journey is as beautiful as you are! GOD BYE TO YOU!!!");
				break;
			}
		}
		// update our lover from blockchain
		if update {
			match poppy.get_chain_account() {
				Ok(lover) => {
					current_lover = lover;
				}
				Err(e) => {println!("couldnt get account: {:?}", e);}
			}
		}
	}
	thread::sleep(Duration::from_millis(300));
	poppy.update_book();
	poppy.book.read();
}

fn is_integer(s: &str) -> bool {
	s.parse::<u64>().is_ok()
}
fn ask_for_input() -> (String, u8) {
	let mut input = String::new();
	stdin().read_line(&mut input).expect("Failed to read");
	let letter: u8 = input.chars().next().unwrap() as u8;
	input.pop();
	(input, letter)
}

fn parse_dream(dream: &str) -> bool{
	let mut rng = rand::rng();
	for _word in dream.split_whitespace() {
		if rng.random_range(1..=100) > 50 {
			return true;
		}
	}
	false
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

