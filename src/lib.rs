use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
	account_info::{next_account_info, AccountInfo},
	entrypoint,
	entrypoint::ProgramResult,
	msg,
	program::invoke,
	program_error::ProgramError,
	pubkey::Pubkey,
	system_instruction,
	sysvar::{rent::Rent, Sysvar},
};
// Program entrypoint
entrypoint!(process_instruction);

pub const NAME: usize = 32;
pub const MAX_SPECIAL: usize = 5;

// Function to route instruction to the correct handler
pub fn process_instruction(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	instruction_data: &[u8],
) -> ProgramResult {
	let instruction = LoverInstruction::unpack(instruction_data)?;
	match instruction {
		LoverInstruction::SetSpecialNumber { new_number } => {
			return set_special_number(program_id, accounts, new_number);
		},
    LoverInstruction::SetName { new_name } => {
			return process_initialize_account(program_id, accounts, new_name);
		},
		LoverInstruction::IncreaseLove => increase_love(program_id, accounts)?,
	}
	Ok(())
}

// Instructions that our program can execute
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum LoverInstruction {
	SetName { new_name: [u8; NAME] },
	SetSpecialNumber { new_number: u64 },
	IncreaseLove,
}

impl LoverInstruction {
	pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
		// Get the instruction varian from the first byte
		let (&variant, rest) = input
			.split_first()
			.ok_or(ProgramError::InvalidInstructionData)?;

		// match instruction type and parse the remaining bytes based on the variant
		match variant {
			0 => {
				// padding name with 0s to make sure it fits in fixed size buffer
				let mut new_name = [0u8; NAME];
				let len = rest.len().min(NAME);
				new_name[..len].copy_from_slice(&rest[..len]);
				if len < NAME {
					new_name[len..].fill(0);
				}
				Ok(Self::SetName { new_name })
			}
			1 => {
				let new_number = u64::from_le_bytes(
					rest.try_into()
						.map_err(|_| ProgramError::InvalidInstructionData)?,
				);
				Ok(Self::SetSpecialNumber { new_number })
			},
			2 => Ok(Self::IncreaseLove),
			_ => Err(ProgramError::InvalidInstructionData),
		}
	}
}

// Initilialize a new Special Numbers Account
fn process_initialize_account(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	new_name: [u8; NAME],
) -> ProgramResult {
	let accounts_iter = &mut accounts.iter();

	let love_account = next_account_info(accounts_iter)?;
	let payer_account = next_account_info(accounts_iter)?;
	let system_program = next_account_info(accounts_iter)?;

	// Size of our counter account
	let account_space = NAME + (MAX_SPECIAL * 8) + 1; // String + Vec meta data + max conent + u8 for current max

	// Calculate minimum balance for rent exemption
	let rent = Rent::get()?;
	let required_lamports = rent.minimum_balance(account_space);

	invoke(
		&system_instruction::create_account(
			payer_account.key,		// Account paying for the new account
			love_account.key,	// Account to be created
			required_lamports,		// Amount of lamports to transfer to the new account
			account_space as u64,	// Size in bytes to allocate for the data field
			program_id,						// Set program owner to our program
		),
		&[
			payer_account.clone(),
			love_account.clone(),
			system_program.clone(),
		],
	)?;
	// Create a new LoverAccount struct with the initial value
	let love_data = Lover {
		name: new_name.clone(),
		special_numbers: [0, 0, 0, 0, 0],
		love: 0,
	};
	// Get a mutable reference to the lover account's data
	let mut account_data = &mut love_account.data.borrow_mut()[..];

	// Serialize the LoverAccount struct into the account's data
	love_data.serialize(&mut account_data)?;
	
	Ok(())
}

fn set_special_number(
	program_id: &Pubkey, 
	accounts: &[AccountInfo], 
	new_number: u64
) -> ProgramResult {
		msg!("setting special number");
		let accounts_iter = &mut accounts.iter();
		let lover_account = next_account_info(accounts_iter)?;
		// Verify account ownership
		if lover_account.owner != program_id {
			return Err(ProgramError::IncorrectProgramId);
		}
		// Mutable borrow the account data
		let mut data = lover_account.data.borrow_mut();
		//Deserialize the account data into our Lover Struct
		match Lover::try_from_slice(&data[..]) {
			Ok(mut lover_data) => {	
				// check to make sure they don't exceed their maximum number of special numbers
				// also that they dont already have this special number
				let mut count = 0;
				for i in 0..MAX_SPECIAL {
					if lover_data.special_numbers[i] != 0 {
						count += 1;
						// we also don't want repeats!
						if lover_data.special_numbers[i] == new_number {
							return Err(CustomError::AlreadySpecial.into());
						}
					} else {
						break;
					}
				}
				if count <= lover_data.love {
					lover_data.special_numbers[lover_data.love as usize] = new_number;
					lover_data.serialize(&mut &mut data[..])?;
				} else {
					return Err(CustomError::NotEnoughLove.into());
				}
			}
			Err(e) => {
				return Err(e.into());
			}
		}
		Ok(())
}

fn increase_love(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
	let accounts_iter = &mut accounts.iter();
	let lover_account = next_account_info(accounts_iter)?;
	// Verify account ownership
	if lover_account.owner != program_id {
		return Err(ProgramError::IncorrectProgramId);
	}
	let mut data = lover_account.data.borrow_mut();
	//Deserialize the account data into our Lover Struct
	match Lover::try_from_slice(&data[..]) {
		Ok(mut lover_data) => {
			if (lover_data.love as usize) < MAX_SPECIAL - 1 {
				lover_data.love += 1;
				lover_data.serialize(&mut &mut data[..])?;
			} else {
				return Err(CustomError::TooMuchLove.into());
			}
		}
		Err(e) => {
			return Err(e.into());
		}
	}
	Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Lover {
	pub name: [u8; NAME],
	pub special_numbers: [u64; MAX_SPECIAL],
	pub love: u8,
}

impl Lover {
	pub fn new() -> Self {
		Self { name: string_to_u8(""), special_numbers: [0,0,0,0,0], love: 0 }
	}

	pub fn count(&self) -> u8 {
		let mut count = 0;
		for i in 0..MAX_SPECIAL {
			if self.special_numbers[i] != 0 {
				count += 1;
			} else {
				break;
			}
		}
		count
	}
}

pub fn u8_to_string(data: [u8; NAME]) -> String {
	let len = data.iter().position(|&b| b == 0).unwrap_or(NAME); // Find null terminator
  String::from_utf8_lossy(&data[..len]).to_string()
}

pub fn string_to_u8(data: &str) -> [u8; NAME] {
	let bytes = data.as_bytes();
	let mut word = [0u8; NAME];
	let len = bytes.len().min(NAME);
	word[..len].copy_from_slice(&bytes[..len]);
	if len < NAME {
		word[len..].fill(0);
	}
	word
}

#[derive(Debug, Clone)]
pub enum CustomError {
	AlreadySpecial = 0,
	NotEnoughLove = 1,
	TooMuchLove = 2,
}

impl From<CustomError> for ProgramError {
    fn from(e: CustomError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
