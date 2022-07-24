#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use frame_support::{pallet_prelude::{*, ValueQuery, OptionQuery}, dispatch::DispatchResult, traits::UnixTime};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::traits::SaturatedConversion;
use pallet_account::{Role, AccountPallet};
use sp_std::vec::Vec;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	pub use super::*;
	pub type VacId = Vec<u8>;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MaxListSize: Get<u32>;
		type UnixTime: UnixTime;
		type AccountInfo: AccountPallet<Self::AccountId>;
	}

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;


	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct VaccineTypeInfo {
	    pub vac_type_id: Option<u32>,
		// TODO: add metadata or hash of metadata
		// pub metadata: Option<Vec<u8>>,
	}

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct VaccineInfo<Account, BoundedAccountList> {
	    pub vac_id: Option<VacId>,
		pub manufacture_id: Option<Account>,
		pub owner_id: Option<Account>,
		pub buyer_id: Option<Account>,
		pub vao_list: BoundedAccountList,
		// true -> buy, false -> not buy
		pub buy_confirm: bool,
		pub vac_type_id: Option<u32>,
		pub max_inoculations_number: u32,
		pub inoculation_count: u32,
	}

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct PassportInfo<Account, BoundedIndexList> {
		pub user_id: Option<Account>,
		pub vac_list: BoundedIndexList,
		pub inoculation_count: u32,
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(bounds(), skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct MovingInfo<T: Config> {
		pub from: Option<T::AccountId>,
		pub to: Option<T::AccountId>,
		pub time: Option<u64>,
	}

	impl<T: Config> MovingInfo<T> {
		pub fn new(from: Option<T::AccountId>, to: Option<T::AccountId>) -> Self {
			MovingInfo { from, to, time: Some(T::UnixTime::now().as_millis().saturated_into::<u64>()) }
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// vaccine ID => VaccineInfo struct
	#[pallet::storage]
	#[pallet::getter(fn vaccines)]
	pub type Vaccines<T: Config> = StorageMap<_, Blake2_128Concat, VacId, VaccineInfo<AccountIdOf<T>, BoundedVec<AccountIdOf<T>, T::MaxListSize>>, OptionQuery>;


	// vaccine ID => MovingInfo struct
	#[pallet::storage]
	#[pallet::getter(fn ownership_tracking)]
	pub type OwnershipTracking<T: Config> = StorageMap<_, Blake2_128Concat, VacId, MovingInfo<T>, OptionQuery>;

	// Account ID => PassportInfo struct
	#[pallet::storage]
	#[pallet::getter(fn vaccine_passports)]
	pub type VaccinePassports<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, PassportInfo<AccountIdOf<T>, BoundedVec<VacId, T::MaxListSize>>, OptionQuery>;

	// (Vaccine ID, Account ID) => true/false(true: used)
	#[pallet::storage]
	#[pallet::getter(fn used_vaccine)]
	pub type UsedVaccine<T: Config> = StorageDoubleMap<_, Blake2_128Concat, VacId, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		RegisterVaccine(VacId),
		TransferVaccine(VacId),
		ReceiveVaccine(T::AccountId, T::AccountId, VacId),
		VaccineOwnershipTransfered(VacId),
		VaccineApproved(VacId, T::AccountId),
		HadVaccination(VacId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotRegisteredVaccine,
		VaccineIsRegistered,
		WrongVaccineOwner,
		NotVaccineBuyer,
		TransferByMyself,
		VaccineAlreadyMine,
		VaccineAlreadyUsed,
		ExceedMaxShotNumber,
		NotSendFinalTransfer,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		// register vaccine information by only manufacture
		#[pallet::weight(10_000)]
		pub fn register_vac_info(origin: OriginFor<T>, vac_id:VacId, vac_type_id: Option<u32>) -> DispatchResult {

			let manufacture = ensure_signed(origin)?;

			// only manufacture
			T::AccountInfo::check_account(&manufacture, Role::VM)?;
			// confirm exist vaccine type
			// ensure!(<VaccineType<T>>::contains_key(vac_type_id.unwrap()), Error::<T>::NotRegisteredVaccineType);


			match Vaccines::<T>::try_get(&vac_id){

				Ok(_) => return Err(Error::<T>::VaccineIsRegistered)?,
				Err(_) => {
					let vac_info = VaccineInfo::<AccountIdOf<T>, BoundedVec<AccountIdOf<T>, T::MaxListSize>>{
						vac_id: Some(vac_id.clone()),
						manufacture_id: Some(manufacture.clone()),
						owner_id: Some(manufacture.clone()),
						buyer_id: None,
						vao_list: Default::default(),
						buy_confirm: false,
						vac_type_id,
						max_inoculations_number: 8,
						inoculation_count: 0,
					};
					// Update storage.     
					<Vaccines<T>>::insert(&vac_id, vac_info);
				}
			};

			Self::transfer_onwership(Some(manufacture), None, vac_id.clone())?;

			// Emit an event.
			Self::deposit_event(Event::RegisterVaccine(vac_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// transfer vaccine by only manufacture and distributer
		#[pallet::weight(10_000)]
		pub fn transfer_vaccine(origin: OriginFor<T>, buyer_id: T::AccountId, vac_id: VacId) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			// only manufacture or distributer
			T::AccountInfo::check_union(&sender, Role::VM, Role::VAD)?;
			T::AccountInfo::check_union(&buyer_id, Role::VM, Role::VAD)?;

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(&vac_id), Error::<T>::NotRegisteredVaccine);

			// confirm buyer is not me
			ensure!(sender != buyer_id, Error::<T>::TransferByMyself);

			// vaccine info のownerがsenderか確認
			let vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			let owner_id = vac_info.owner_id.unwrap();
			ensure!(owner_id == sender, Error::<T>::WrongVaccineOwner);
			// confirm vaccine not used
			let count = vac_info.inoculation_count;
			ensure!(count == 0, Error::<T>::VaccineAlreadyUsed);

			// 送る structとstorageの更新
			let mut new_vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			new_vac_info.buyer_id = Some(buyer_id);
			new_vac_info.buy_confirm = false;
			<Vaccines<T>>::insert(&vac_id, new_vac_info);
			
			// Emit an event.
			Self::deposit_event(Event::TransferVaccine(vac_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// receive vaccine by only manufacture and distributer
		#[pallet::weight(10_000)]
		pub fn receive_vaccine(origin: OriginFor<T>, sender: T::AccountId, vac_id: VacId) -> DispatchResult {
			let receiver = ensure_signed(origin)?;

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(&vac_id), Error::<T>::NotRegisteredVaccine);

			// only manufacture or distributer
			T::AccountInfo::check_union(&receiver, Role::VM, Role::VAD)?;

			// only specified receiver
			let vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			let buyer_id = vac_info.buyer_id.unwrap();
			ensure!(receiver == buyer_id, Error::<T>::NotVaccineBuyer);
			// confirm vaccine not used
			let count = vac_info.inoculation_count;
			ensure!(count == 0, Error::<T>::VaccineAlreadyUsed);
			// confirm vaccine will not transfer
			let confirmation = <Vaccines<T>>::get(&vac_id).unwrap().buy_confirm;
			ensure!(!confirmation, Error::<T>::VaccineAlreadyMine);
			// confirm correct vaccine owner
			let owner = <Vaccines<T>>::get(&vac_id).unwrap().owner_id.unwrap();
			ensure!(owner == sender, Error::<T>::WrongVaccineOwner);

			// update struct and storage 
			let mut new_vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			new_vac_info.owner_id = Some(receiver.clone());
			new_vac_info.buy_confirm = true;
			
			<Vaccines<T>>::insert(&vac_id, new_vac_info);

			Self::transfer_onwership(Some(sender.clone()), Some(receiver.clone()), vac_id.clone())?;

			// Emit an event.
			Self::deposit_event(Event::ReceiveVaccine(receiver, sender, vac_id));

			Ok(())
		}

		// approve vaccine by only approved organization
		#[pallet::weight(10_000)]
		pub fn approve_vaccine(origin: OriginFor<T>, vac_id: VacId) -> DispatchResult {

			let organization = ensure_signed(origin)?;

			// only approved organization
			T::AccountInfo::check_account(&organization, Role::VAO)?;
			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(&vac_id), Error::<T>::NotRegisteredVaccine);
			//TODO: confirm dont double approve by one organization

			// confirm vaccine not used
			let vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			let count = vac_info.inoculation_count;
			ensure!(count == 0, Error::<T>::VaccineAlreadyUsed);

			// upddate struct(vao list)
			let mut new_vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			new_vac_info.vao_list.try_push(organization.clone());
			// Update storage.
			<Vaccines<T>>::insert(&vac_id, new_vac_info);

			// Emit an event.
			Self::deposit_event(Event::VaccineApproved(vac_id, organization));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// finally transfer vaccine to user
		#[pallet::weight(10_000)]
		pub fn transfer_get_vaccine_right(origin: OriginFor<T>, user_id: T::AccountId, vac_id: VacId) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			// only manufacture or distributer
			T::AccountInfo::check_union(&sender, Role::VM, Role::VAD)?;

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(&vac_id), Error::<T>::NotRegisteredVaccine);

			// confirm vaccine not duplicated used
			ensure!(!<UsedVaccine<T>>::get(&vac_id, &user_id), Error::<T>::VaccineAlreadyUsed);

			// confirm buyer is not me
			ensure!(sender != user_id, Error::<T>::TransferByMyself);

			// vaccine info のownerがsenderか確認
			let vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			let owner_id = vac_info.owner_id.unwrap();
			ensure!(owner_id == sender, Error::<T>::WrongVaccineOwner);

			// 送る structとstorageの更新
			let mut new_vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			new_vac_info.buyer_id = Some(user_id.clone());
			new_vac_info.buy_confirm = false;
			// confirm inoculation count dont reach max number
			ensure!(new_vac_info.inoculation_count < new_vac_info.max_inoculations_number, Error::<T>::ExceedMaxShotNumber);
			new_vac_info.inoculation_count += 1;
			<Vaccines<T>>::insert(&vac_id, new_vac_info);

			// register vaccine is used
			<UsedVaccine<T>>::insert(&vac_id, user_id, true);
			
			// Emit an event.
			Self::deposit_event(Event::TransferVaccine(vac_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// user confirm vaccine
		#[pallet::weight(10_000)]
		pub fn confirm_vaccine(origin: OriginFor<T>, vac_owner: Option<T::AccountId>,  vac_id: VacId) -> DispatchResult {

			let user = ensure_signed(origin)?;

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(&vac_id), Error::<T>::NotRegisteredVaccine);

			// confirm vaccine owner
			let owner = <Vaccines<T>>::get(&vac_id).unwrap().owner_id;
			let confirmation = <Vaccines<T>>::get(&vac_id).unwrap().buy_confirm;
			// only specified receiver
			let vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			let buyer_id = vac_info.buyer_id.unwrap();
			ensure!(user == buyer_id, Error::<T>::NotVaccineBuyer);
			// confirm vaccine will not transfer
			ensure!(!confirmation, Error::<T>::VaccineAlreadyMine);
			// confirm vaccine correct owner
			ensure!(owner == vac_owner, Error::<T>::WrongVaccineOwner);

			// confirm send_final_transfer is sended to me?
			ensure!(<UsedVaccine<T>>::get(&vac_id, &user), Error::<T>::NotSendFinalTransfer);

			// update struct and storage 
			let mut new_vac_info = <Vaccines<T>>::get(&vac_id).unwrap();
			// delete to multiple shot
			// new_vac_info.owner_id = Some(user.clone());
			// new_vac_info.buy_confirm = true;
			<Vaccines<T>>::insert(&vac_id, new_vac_info);

			// issuing vaccine passport
			Self::register_vac_pass(user.clone(), vac_id.clone())?;


			Self::transfer_onwership(Some(vac_owner.unwrap().clone()), Some(user.clone()), vac_id.clone())?;

			// Emit an event.
			Self::deposit_event(Event::HadVaccination(vac_id, user));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
/*----------------------------------------------helper function ------------------------------------------------- */
	impl<T: Config> Pallet<T> {

		pub fn transfer_onwership(from: Option<T::AccountId>, to: Option<T::AccountId>, vac_id: VacId) -> DispatchResult {
			let time = MovingInfo::<T>::new(from.clone(), to.clone());
			<OwnershipTracking<T>>::insert(&vac_id, &time);

			// Emit an event.
			Self::deposit_event(Event::VaccineOwnershipTransfered(vac_id));

			Ok(())
		}


		pub fn register_vac_pass(registrant: T::AccountId, vac_id: VacId) -> DispatchResult {
			match VaccinePassports::<T>::try_get(&registrant){
				Ok(_) => {
				},
				Err(_) => {
					let vac_pass = PassportInfo::<AccountIdOf<T>, BoundedVec<VacId, T::MaxListSize>>{
						user_id: Some(registrant.clone()),
						vac_list: Default::default(),
						inoculation_count: 0,
					};

					// Update storage.     
					<VaccinePassports<T>>::insert(&registrant, vac_pass);
				}
			};

			// register vaccine list
			let mut passport = Self::vaccine_passports(&registrant).unwrap();
			passport.vac_list.try_push(vac_id);
			passport.inoculation_count += 1;

			// Update storage.     
			<VaccinePassports<T>>::insert(&registrant, passport);


			Ok(())
		}

	}
}
