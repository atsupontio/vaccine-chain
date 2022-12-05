#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

use sp_std::vec::Vec;
#[cfg(feature = "std")]
use serde::{ser::Error as SerdeError, Deserialize, Deserializer, Serialize, Serializer};

// pub type RoleId = [u8; 36];
pub type RoleId = Vec<u8>;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub trait AccountPallet {
	fn check_claim_account(claimer: &RoleId, role: Role) -> DispatchResult;
	fn check_account(who: &RoleId, role: Role) -> DispatchResult;
	fn check_union(who: &RoleId, role1: Role, role2: Role) -> DispatchResult;
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

	#[derive(
		Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo,
	)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Role {
		SYSMAN,
		VM,
		VAO,
		VAD,
		USER,
	}

	#[derive(
		Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo,
	)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum RoleStatus {
		Approved,
		Revoked,
		Pending,
	}

	impl Default for RoleStatus {
		fn default() -> Self {
			Self::Pending
		}
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct Account {
		role: Role,
		status: RoleStatus,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// Account ID => Account struct
	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T> = StorageMap<_, Blake2_128Concat, RoleId, Account, OptionQuery>;

	// Account ID => Role
	#[pallet::storage]
	#[pallet::getter(fn account_role)]
	pub type AccountRole<T: Config> =
		StorageMap<_, Blake2_128Concat, RoleId, T::AccountId, OptionQuery>;

	/// Store admin user account for special purpose
	#[pallet::storage]
	#[pallet::getter(fn system_manager)]
	pub type SystemManager<T: Config> = StorageMap<_, Twox64Concat, RoleId, bool, OptionQuery>;

	// Alice is sysman by default
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub genesis_account: Vec<Vec<u8>>,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { genesis_account: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			for role_id in &self.genesis_account {
				let account = Account { role: Role::SYSMAN, status: RoleStatus::Approved };
				<Accounts<T>>::insert(role_id, account);
				SystemManager::<T>::insert(role_id, true);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {
		Claimed(RoleId),
		Approved(RoleId),
		AccountRegisted(RoleId),
		RemoveSystem(RoleId),
		AddSystem(RoleId),
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
		NotFoundRole,
		PermissionDeny,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn approve_role(
			origin: OriginFor<T>,
			system: RoleId,
			target: RoleId,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(Self::only_system(system.clone()), Error::<T>::PermissionDeny);
			// only sysman execute
			Self::check_account(&system, Role::SYSMAN)?;

			let mut account = <Accounts<T>>::get(&target).unwrap();

			ensure!(account.status != RoleStatus::Approved, Error::<T>::InvalidStatus);

			account.status = RoleStatus::Approved;

			// Update storage.
			<Accounts<T>>::insert(&target, &account);
			SystemManager::<T>::insert(target.clone(), true);

			// Emit an event.
			Self::deposit_event(Event::Approved(target));
			// Return a successful DispatchResultWithPostInfo  
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn register_account(
			origin: OriginFor<T>,
			role_id: RoleId,
			role: Role,
		) -> DispatchResult {
			ensure_root(origin)?;
			match <Accounts<T>>::try_get(&role_id) {
				Err(_) => {
					<Accounts<T>>::insert(&role_id, Account { role, status: Default::default() });
				},
				Ok(_) => Err(Error::<T>::AlreadyRegistered)?,
			}
			// Return a successful DispatchResultWithPostInfo
			Self::deposit_event(Event::AccountRegisted(role_id));
			Ok(())
		}

		/// add admin for special purposes
		#[pallet::weight(10_000)]
		pub fn add_system(origin: OriginFor<T>, system: RoleId, user: RoleId) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			if !Self::only_system(system) {
				return Err(Error::<T>::PermissionDeny)?
			}

			SystemManager::<T>::insert(&user, true);
			Self::deposit_event(Event::AddSystem(user));
			Ok(())
		}

		/// remove admin for special purposes
		#[pallet::weight(10_000)]
		pub fn remove_system(origin: OriginFor<T>, system: RoleId, user: RoleId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			if !Self::only_system(system) {
				return Err(Error::<T>::PermissionDeny)?
			}
			SystemManager::<T>::remove(&user);
			Self::deposit_event(Event::RemoveSystem(user));

			Ok(())
		}
	}

	/* ----------------------------------------------helper function
	 * ------------------------------------------------- */
	impl<T: Config> AccountPallet for Pallet<T> {
		fn check_claim_account(claimer: &RoleId, role: Role) -> DispatchResult {
			let account = <Accounts<T>>::get(claimer).unwrap();
			match account.role {
				a if a == role => Err(Error::<T>::AlreadyClaimed)?,
				Role::USER => match account.status {
					RoleStatus::Approved => Err(Error::<T>::AlreadyApproved)?,
					RoleStatus::Revoked => Err(Error::<T>::AlreadyRevoked)?,
					RoleStatus::Pending => Err(Error::<T>::AlreadyClaimed)?,
				},
				_ => Err(Error::<T>::InvalidRole)?,
			}
		}

		fn check_account(who: &RoleId, role: Role) -> DispatchResult {
			let account = <Accounts<T>>::get(who).unwrap();
			log::info!("Here ");
			match account.role {

				a if a == role => match account.status {
					
					RoleStatus::Approved => return Ok(()),
					RoleStatus::Revoked => Err(Error::<T>::AlreadyRevoked)?,
					RoleStatus::Pending => Err(Error::<T>::NotApproved)?,
				},
				_ => {
					log::info!("Here 3");
					return Err(Error::<T>::InvalidRole)?;},
			}
		}

		fn check_union(who: &RoleId, role1: Role, role2: Role) -> DispatchResult {
			let account = <Accounts<T>>::get(who).unwrap();
			match account.role {
				a if a == role1 => match account.status {
					RoleStatus::Approved => return Ok(()),
					RoleStatus::Revoked => Err(Error::<T>::AlreadyRevoked)?,
					RoleStatus::Pending => Err(Error::<T>::NotApproved)?,
				},
				b if b == role2 => match account.status {
					RoleStatus::Approved => return Ok(()),
					RoleStatus::Revoked => Err(Error::<T>::AlreadyRevoked)?,
					RoleStatus::Pending => Err(Error::<T>::NotApproved)?,
				},
				_ => Err(Error::<T>::InvalidRole)?,
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn only_system(user: RoleId) -> bool {
		SystemManager::<T>::get(user).unwrap_or(false)
	}
}

pub struct UserId([u8; 36]);

#[cfg(feature = "std")]
impl Serialize for UserId {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		std::str::from_utf8(&self.0)
			.map_err(|e| S::Error::custom(format!("Debug buffer contains invalid UTF8 :{}", e)))?
			.serialize(serializer)
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for UserId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;

		let bytes: [u8; 36] = s.try_into().expect("Slice with incorrect length");

		Ok(UserId::from(bytes))
		// Ok(String::deserialize(deserializer)?
		// 	.as_bytes()
		// 	.try_into()
		// 	.expect("Slice with incorrect length"))
	}
}

#[cfg(feature = "std")]
impl From<[u8; 36]> for UserId {
	fn from(item: [u8; 36]) -> Self {
		UserId(item)
	}
}
