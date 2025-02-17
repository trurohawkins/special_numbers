#[cfg(test)]
mod test {
		use special_numbers::{
			process_instruction,
			Lover,
			string_to_u8,
		};
    use solana_program_test::*;
    use solana_sdk::{
			signature::{Keypair, Signer},
			transaction::Transaction,
			instruction::AccountMeta,
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
				//let new_number: u64 = 69;
				let new_name = "TurtleGod";
				let new_number: u64 = 69;
				//let mut lover_data = LoverInstruction::SetSpecialNumber { new_number };
				let name = string_to_u8(new_name);

				let mut lover_data = vec![0];
				lover_data.extend_from_slice(&name);//vec![];

        //lover_data.extend_from_slice(&new_number.to_le_bytes());
        // Create the instruction to invoke the program
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
				
				println!("checkin accoutn data");
				// check account data
				let account = banks_client
					.get_account(lover_keypair.pubkey())
					.await.expect("Failed to get lover account");
				println!("double check");
				if let Some(account_data) = account {
					match Lover::try_from_slice(&account_data.data) {
						Ok(lover) => {
							//msg!("You have a soecial connection with {}", u8_to_string(lover.name));
							assert_eq!(lover.name, name);
							assert_eq!(lover.special_number, 0);
						},
						Err(e) => eprintln!("Failed to deserialize: {}", e),
					}
				}
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
					.expect("Failed tp get lover account");

				if let Some(account_data) = account {
					let lover: Lover = Lover::try_from_slice(&account_data.data)
						.expect("Failed t odeserialize counter data");
					assert_eq!(lover.special_number, 69);
				}
        // Process the transaction
        //let transaction_result = banks_client.process_transaction(transaction).await;
        //assert!(transaction_result.is_ok());
    }
}

