#[cfg(test)]
mod test {
	use special_numbers::{
		process_instruction,
			Lover,
			string_to_u8,
			CustomError,
	};
	use solana_program_test::*;
	use solana_sdk::{
		signature::{Keypair, Signer},
			transaction::{Transaction, TransactionError},
			instruction::{AccountMeta, InstructionError},
			system_program,
	};
	use borsh::de::BorshDeserialize;
	use solana_program::pubkey::Pubkey;

#[tokio::test]
	async fn test_special_numbers() {
		let program_id = Pubkey::new_unique();
		let (mut banks_client, payer, recent_blockhash) =
			ProgramTest::new("special_numbers", 
					program_id,
					processor!(process_instruction))
			.start()
			.await;
		let lover_keypair = Keypair::new();

		// test initialization
		let new_name = "TurtleGod";
		let name = string_to_u8(new_name);
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
		transaction.sign(&[&payer, &lover_keypair], recent_blockhash);
		banks_client.process_transaction(transaction).await.unwrap();

		// check account data
		let account = banks_client
			.get_account(lover_keypair.pubkey())
			.await.expect("Failed to get lover account");
		if let Some(account_data) = account {
			match Lover::try_from_slice(&account_data.data) {
				Ok(lover) => {
					assert_eq!(lover.name, name);
				},
					Err(e) => eprintln!("Failed to deserialize: {}", e),
			}
		}

		// check adding 1st special number
		let new_number: u64 = 69;
		let mut number_data = vec![1];
		number_data.extend_from_slice(&new_number.to_le_bytes());//vec![];
		let number_instruction = solana_program::instruction::Instruction::new_with_bytes(
				program_id,
				&number_data,
				vec![AccountMeta::new(lover_keypair.pubkey(), true)]
				);
		let mut transaction =
			Transaction::new_with_payer(&[number_instruction], Some(&payer.pubkey()));
		transaction.sign(&[&payer, &lover_keypair], recent_blockhash);

		banks_client.process_transaction(transaction).await.unwrap();

		let account = banks_client
			.get_account(lover_keypair.pubkey())
			.await
			.expect("Failed to get lover account");
		if let Some(account_data) = account {
			let lover: Lover = Lover::try_from_slice(&account_data.data)
				.expect("Failed to deserialize counter data");
			assert_eq!(lover.special_numbers, [69, 0, 0, 0, 0]);
		}
		// error test, checking adding number beyond limit of current max
		let second_number: u64 = 420;
		let mut number_data = vec![1];
		number_data.extend_from_slice(&second_number.to_le_bytes());//vec![];
		let number_instruction = solana_program::instruction::Instruction::new_with_bytes(
				program_id,
				&number_data,
				vec![AccountMeta::new(lover_keypair.pubkey(), true)]
				);
		let mut transaction =
			Transaction::new_with_payer(&[number_instruction], Some(&payer.pubkey()));
		transaction.sign(&[&payer, &lover_keypair], recent_blockhash);

		let result = banks_client.process_transaction(transaction.clone()).await;
		match result {
			Err(BanksClientError::TransactionError(tx_error)) => {
				if let TransactionError::InstructionError(_, instruction_error) = tx_error {
					assert_eq!(instruction_error, InstructionError::Custom(CustomError::NotEnoughLove as u32), "No enough love for 2nd special number");	
				} else {
					panic!("unexpected error {:?}", tx_error);
				}
			}
			err => panic!("transaction went through, but it should have been denied! {:?}", err),
		}
		// testing increase in max LOVE!
		let increase_data = vec![2];
		let increase_instruction = solana_program::instruction::Instruction::new_with_bytes(
				program_id,
				&increase_data,
				vec![AccountMeta::new(lover_keypair.pubkey(), true)]
				);
		let mut inc_tx =
			Transaction::new_with_payer(&[increase_instruction], Some(&payer.pubkey()));
		inc_tx.sign(&[&payer, &lover_keypair], recent_blockhash);

		banks_client.process_transaction(inc_tx).await.unwrap();
		let account = banks_client
			.get_account(lover_keypair.pubkey())
			.await
			.expect("Failed to get lover account");
		if let Some(account_data) = account {
			let lover: Lover = Lover::try_from_slice(&account_data.data)
				.expect("Failed t odeserialize counter data");
			assert_eq!(lover.love, 1);
		}
		// testing this time the special number should go through!
		let second_try: u64 = 42; // has to be different because in this test environemtn if you try to send the same transaction, it resends the old? or something, the result comes back teh same as before
		let mut number_data = vec![1];
		number_data.extend_from_slice(&second_try.to_le_bytes());//vec![];
		let number_instruction = solana_program::instruction::Instruction::new_with_bytes(
				program_id,
				&number_data,
				vec![AccountMeta::new(lover_keypair.pubkey(), true)]
				);
		let mut transaction =
			Transaction::new_with_payer(&[number_instruction], Some(&payer.pubkey()));
		transaction.sign(&[&payer, &lover_keypair], recent_blockhash);

		banks_client.process_transaction(transaction).await.unwrap();
		let account = banks_client
			.get_account(lover_keypair.pubkey())
			.await
			.expect("Failed to get lover account");
		if let Some(account_data) = account {
			let lover: Lover = Lover::try_from_slice(&account_data.data)
				.expect("Failed to deserialize counter data");
			assert_eq!(lover.special_numbers[1], 42);
		}
		/*
		println!("sending 69 again");
		let again_with_this: u64 = 69;
		let mut number_data = vec![1];
		number_data.extend_from_slice(&again_with_this.to_le_bytes());//vec![];
		let number_instruction = solana_program::instruction::Instruction::new_with_bytes(
				program_id,
				&number_data,
				vec![AccountMeta::new(lover_keypair.pubkey(), true)]
				);
		let mut transaction =
			Transaction::new_with_payer(&[number_instruction], Some(&payer.pubkey()));
		transaction.sign(&[&payer, &lover_keypair], recent_blockhash);

	//	let result = banks_client.process_transaction(transaction.clone()).await.unwrap();
		
		let result = banks_client.process_transaction(transaction.clone()).await;
		match result {
			Err(BanksClientError::TransactionError(tx_error)) => {
				if let TransactionError::InstructionError(_, instruction_error) = tx_error {
					assert_eq!(instruction_error, InstructionError::Custom(CustomError::AlreadySpecial as u32), "This number is already special to you");	
				} else {
					panic!("unexpected error {:?}", tx_error);
				}
			}
			err => panic!("transaction went through, but it should have been denied! {:?}", err),
		}
		*/
	}
}

