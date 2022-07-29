#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

use frame_support::{pallet_prelude::*, dispatch::DispatchResult, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use scale_info::TypeInfo;
	use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub trait AccountPallet<AccountId>{
	fn check_claim_account(claimer: &AccountId, role: Role) -> DispatchResult;
	fn check_account(who: &AccountId, role: Role) -> DispatchResult;
	fn check_union(who: &AccountId, role1: Role, role2: Role) -> DispatchResult;
}

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	pub use frame_support::inherent::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MaxListSize: Get<u32>;
		type UnixTime: UnixTime;
	}

	pub type VaccineTypeIndex = u32;
	pub type VaccineIndex = u32;
	pub type RecognitionId = u32;
	pub type String = Vec<u8>;

	#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Role {
		SYSMAN,
		GOV,
		VM,
		VAO,
		VAD,
		USER,
	}

	impl Default for Role {
		fn default() -> Self {
			Self::USER
		}
	}

	#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum RoleStatus {
		Approved,
		Revoked,
		Pending,
		None,
	}

	impl Default for RoleStatus {
		fn default() -> Self {
			Self::None
		}
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct Account<AccountId> {
		id: AccountId,
		name: String,
		recognition_id: RecognitionId,
		role: Role,
		status: RoleStatus,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Account ID => Account struct
	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Account<T::AccountId>, OptionQuery>;

	// Account ID => Role
	#[pallet::storage]
	#[pallet::getter(fn account_role)]
	pub type AccountRole<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Role, OptionQuery>;

	// Alice is sysman by default
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub genesis_account: Vec<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { genesis_account: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for account_id in &self.genesis_account {
				let mut account = Account::<T::AccountId> {
					id: account_id.clone(),
					name: 
					role: Role::SYSMAN,
					status: RoleStatus::Approved,
				};
				account.role = Role::SYSMAN;
				account.status = RoleStatus::Approved;
				<Accounts<T>>::insert(account_id, account);
			}
		}
	}


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Claimed(T::AccountId),
		Approved(T::AccountId),
		AccountRegisted(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyClaimed,
		AlreadyApproved,
		AlreadyRegistered,
		AlreadyRevoked,
		NotClaimed,
		NotApproved,
		InvalidRole,
		InvalidStatus,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
	
		#[pallet::weight(10_000)]
		pub fn claim_role(origin: OriginFor<T>, role: Role) -> DispatchResult {

			let claimer = ensure_signed(origin)?;

			// cannot claim user role
			ensure!(role != Role::USER, Error::<T>::InvalidRole);

			Self::check_claim_account(&claimer, role.clone())?;

			let mut account = <Accounts<T>>::get(&claimer).unwrap();
			account.status = RoleStatus::Pending;
			account.role = role;
			// Update storage.
			<Accounts<T>>::insert(&claimer, account);

			// Emit an event.
			Self::deposit_event(Event::Claimed(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve_role(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			// only sysman execute
			Self::check_account(&sender, Role::SYSMAN)?;

			let mut account = <Accounts<T>>::get(&target).unwrap();

			ensure!(account.status != RoleStatus::None, Error::<T>::InvalidStatus);

			account.status = RoleStatus::Approved;

			// Update storage.
			<Accounts<T>>::insert(&target, &account);
			<AccountRole<T>>::insert(&target, &account.role);

			// Emit an event.
			Self::deposit_event(Event::Approved(target));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn register(origin: OriginFor<T>, name: String, recognition_id: RecognitionId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			match <Accounts<T>>::try_get(&who) {
				Err(_) => {
					<Accounts<T>>::insert(
						&who,
						Account {
							id: who.clone(),
							name,
							recognition_id,
							role: Default::default(),
							status: Default::default(),
						},
					);
				},
				Ok(_) => Err(Error::<T>::AlreadyRegistered)?,
			}
			// Return a successful DispatchResultWithPostInfo
			Self::deposit_event(Event::AccountRegisted(who));
			Ok(())
		}
	}

  /*----------------------------------------------helper function ------------------------------------------------- */
	impl<T: Config> AccountPallet<T::AccountId> for Pallet<T> {

		fn check_claim_account(claimer: &T::AccountId, role: Role) -> DispatchResult {

			let account = <Accounts<T>>::get(claimer).unwrap();
			match account.role {
				a if a == role => Err(Error::<T>::AlreadyClaimed)?,
				Role::USER => {
					match account.status {
						RoleStatus::Approved => Err(Error::<T>::AlreadyApproved)?,
						RoleStatus::Revoked => Err(Error::<T>::AlreadyRevoked)?,
						RoleStatus::Pending => Err(Error::<T>::AlreadyClaimed)?,
						RoleStatus::None => return Ok(()),
					}
				},
				_ => Err(Error::<T>::InvalidRole)?,
			}
		}

		fn check_account(who: &T::AccountId, role: Role) -> DispatchResult {
			let account = <Accounts<T>>::get(who).unwrap();
			match account.role {
				a if a == role => {
					match account.status {
						RoleStatus::Approved => return Ok(()),
						RoleStatus::Revoked => Err(Error::<T>::AlreadyRevoked)?,
						RoleStatus::Pending => Err(Error::<T>::NotApproved)?,
						RoleStatus::None => Err(Error::<T>::NotClaimed)?,
					}
				},
				_ => Err(Error::<T>::InvalidRole)?,
			}
		}

		fn check_union(who: &T::AccountId, role1: Role, role2: Role) -> DispatchResult {
			let account = <Accounts<T>>::get(who).unwrap();
			match account.role {
				a if a == role1 => {
					match account.status {
						RoleStatus::Approved => return Ok(()),
						RoleStatus::Revoked => Err(Error::<T>::AlreadyRevoked)?,
						RoleStatus::Pending => Err(Error::<T>::NotApproved)?,
						RoleStatus::None => Err(Error::<T>::NotClaimed)?,
					}
				},
				b if b == role2 => {
					match account.status {
						RoleStatus::Approved => return Ok(()),
						RoleStatus::Revoked => Err(Error::<T>::AlreadyRevoked)?,
						RoleStatus::Pending => Err(Error::<T>::NotApproved)?,
						RoleStatus::None => Err(Error::<T>::NotClaimed)?,
					}
				},
				_ => Err(Error::<T>::InvalidRole)?,
			}
		}
	}
}
