#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
use frame_support::{pallet_prelude::{*, ValueQuery}, dispatch::DispatchResult};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use scale_info::TypeInfo;
	use serde::{Deserialize, Serialize};
	use frame_support::traits::UnixTime;
	use sp_runtime::traits::SaturatedConversion;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MaxListSize: Get<u32>;
		type UnixTime: UnixTime;
	}

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub type VaccineTypeIndex = u32;
	pub type VaccineIndex = u32;

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct VaccineTypeInfo {
	    pub vac_type_id: Option<u32>,
		// TODO: add metadata or hash of metadata
		// pub metadata: Option<Vec<u8>>,
	}

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct VaccineInfo<Account, BoundedAccountList> {
	    pub vac_id: Option<u32>,
		pub manufacture_id: Option<Account>,
		pub owner_id: Option<Account>,
		pub buyer_id: Option<Account>,
		pub vao_list: BoundedAccountList,
		// true -> buy, false -> not buy
		pub buy_confirm: bool,
		pub vac_type_id: Option<u32>,
		pub max_inoculations_number: u32,
		pub inoculation_count: u32,
		pub approved_time: Option<u64>,
	}

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct PassportInfo<Account, BoundedIndexList> {
		pub user_id: Option<Account>,
		pub vac_list: BoundedIndexList,
		pub inoculation_count: u32,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


	// only claim role account
	#[pallet::storage]
	#[pallet::getter(fn pending_gov)]
	pub type PendingGOV<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pending_sys_man)]
	pub type PendingSYSMAN<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pending_vm)]
	pub type PendingVM<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pending_vao)]
	pub type PendingVAO<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pending_vad)]
	pub type PendingVAD<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;


	// approved account
	#[pallet::storage]
	#[pallet::getter(fn gov)]
	pub type GOV<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sys_man)]
	pub type SYSMAN<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vm)]
	pub type VM<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vao)]
	pub type VAO<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vad)]
	pub type VAD<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vaccine_type)]
	pub type VaccineType<T: Config> = StorageMap<_, Blake2_128Concat, u32, VaccineTypeInfo, OptionQuery>;

	// vaccine ID => VaccineInfo struct
	#[pallet::storage]
	#[pallet::getter(fn vaccines)]
	pub type Vaccines<T: Config> = StorageMap<_, Blake2_128Concat, u32, VaccineInfo<AccountIdOf<T>, BoundedVec<AccountIdOf<T>, T::MaxListSize>>, OptionQuery>;

	#[pallet::type_value]
	pub fn InitialVaccineTypeCount<T: Config>() -> u32 { 1u32 }

	// Vaccine type ID (initial value: 1)
	#[pallet::storage]
	#[pallet::getter(fn vaccine_type_count)]
	pub type VaccineTypeCount<T: Config> = StorageValue<_, VaccineTypeIndex, ValueQuery, InitialVaccineTypeCount<T>>;

	#[pallet::type_value]
	pub fn InitialVaccineCount<T: Config>() -> u32 { 1u32 }

	// Vaccine ID(Initial Value: 1)
	#[pallet::storage]
	#[pallet::getter(fn vaccine_count)]
	pub type VaccineCount<T: Config> = StorageValue<_, VaccineIndex, ValueQuery, InitialVaccineCount<T>>;

	// Account ID => PassportInfo struct
	#[pallet::storage]
	#[pallet::getter(fn vaccine_passports)]
	pub type VaccinePassports<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, PassportInfo<AccountIdOf<T>, BoundedVec<u32, T::MaxListSize>>, OptionQuery>;

	// (Vaccine ID, Account ID) => true/false(true: used)
	#[pallet::storage]
	#[pallet::getter(fn used_vaccine)]
	pub type UsedVaccine<T: Config> = StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

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
				<SYSMAN<T>>::insert(account_id, true);
			}
		}
	}


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Claimed(T::AccountId),
		Approved(T::AccountId),
		RegisterVaccineType(u32),
		RegisterVaccine(u32),
		RequestVaccine(u32),
		TransferVaccine(u32),
		ReceiveVaccine(T::AccountId, T::AccountId, u32),
		VaccineApproved(u32, T::AccountId),
		HadVaccination(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyClaimed,
		AlreadyApproved,
		NotClaimed,
		NotSysMan,
		NotManufacture,
		NotOrganization,
		NotRegisteredVaccineType,
		NotRegisteredVaccine,
		VaccineIsRegistered,
		WrongVaccineOwner,
		NotVaccineBuyer,
		BuyByYourself,
		NotVaccineTransfered,
		VaccineWillTransfer,
		VaccineAlreadyUsed,
		ExceedMaxShotNumber,
		NotSendFinalTransfer,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

/* -----------------------------------------------claim role function ----------------------------------------- */		
		#[pallet::weight(10_000)]
		pub fn claim_sys_man(origin: OriginFor<T>) -> DispatchResult {

			let claimer = ensure_signed(origin)?;

			ensure!(!Self::pending_sys_man(&claimer), Error::<T>::AlreadyClaimed);
			ensure!(!Self::sys_man(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<PendingSYSMAN<T>>::insert(&claimer, true);

			// Emit an event.
			Self::deposit_event(Event::Claimed(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn claim_vm(origin: OriginFor<T>) -> DispatchResult {

			let claimer = ensure_signed(origin)?;

			ensure!(!Self::pending_vm(&claimer), Error::<T>::AlreadyClaimed);
			ensure!(!Self::vm(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<PendingVM<T>>::insert(&claimer, true);

			// Emit an event.
			Self::deposit_event(Event::Claimed(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn claim_gov(origin: OriginFor<T>) -> DispatchResult {

			let claimer = ensure_signed(origin)?;

			ensure!(!Self::pending_gov(&claimer), Error::<T>::AlreadyClaimed);
			ensure!(!Self::gov(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<PendingGOV<T>>::insert(&claimer, true);

			// Emit an event.
			Self::deposit_event(Event::Claimed(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn claim_vao(origin: OriginFor<T>) -> DispatchResult {

			let claimer = ensure_signed(origin)?;

			ensure!(!Self::pending_vao(&claimer), Error::<T>::AlreadyClaimed);
			ensure!(!Self::vao(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<PendingVAO<T>>::insert(&claimer, true);

			// Emit an event.
			Self::deposit_event(Event::Claimed(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn claim_vad(origin: OriginFor<T>) -> DispatchResult {

			let claimer = ensure_signed(origin)?;

			ensure!(!Self::pending_vad(&claimer), Error::<T>::AlreadyClaimed);
			ensure!(!Self::vad(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<PendingVAD<T>>::insert(&claimer, true);

			// Emit an event.
			Self::deposit_event(Event::Claimed(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	/* ----------------------------------------------------------------------------------------------------- */


	/* -------------------------------- approve role function (executed by only sysman) ---------------------*/
		#[pallet::weight(10_000)]
		pub fn approve_sys_man(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let claimer = target;

			ensure!(Self::sys_man(sender), Error::<T>::NotSysMan);

			ensure!(Self::pending_sys_man(&claimer), Error::<T>::NotClaimed);
			ensure!(!Self::sys_man(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<SYSMAN<T>>::insert(&claimer, true);
			<PendingSYSMAN<T>>::insert(&claimer, false);

			// Emit an event.
			Self::deposit_event(Event::Approved(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve_vm(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let claimer = target;

			ensure!(Self::sys_man(sender), Error::<T>::NotSysMan);

			ensure!(Self::pending_vm(&claimer), Error::<T>::NotClaimed);
			ensure!(!Self::vm(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<VM<T>>::insert(&claimer, true);
			<PendingVM<T>>::insert(&claimer, false);

			// Emit an event.
			Self::deposit_event(Event::Approved(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve_gov(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let claimer = target;

			ensure!(Self::sys_man(sender), Error::<T>::NotSysMan);

			ensure!(Self::pending_gov(&claimer), Error::<T>::NotClaimed);
			ensure!(!Self::gov(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<GOV<T>>::insert(&claimer, true);
			<PendingGOV<T>>::insert(&claimer, false);

			// Emit an event.
			Self::deposit_event(Event::Approved(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve_vao(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let claimer = target;

			ensure!(Self::sys_man(sender), Error::<T>::NotSysMan);

			ensure!(Self::pending_vao(&claimer), Error::<T>::NotClaimed);
			ensure!(!Self::vao(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<VAO<T>>::insert(&claimer, true);
			<PendingVAO<T>>::insert(&claimer, false);

			// Emit an event.
			Self::deposit_event(Event::Approved(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve_vad(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let claimer = target;

			ensure!(Self::sys_man(sender), Error::<T>::NotSysMan);

			ensure!(Self::pending_vad(&claimer), Error::<T>::NotClaimed);
			ensure!(!Self::vad(&claimer), Error::<T>::AlreadyApproved);

			// Update storage.
			<VAD<T>>::insert(&claimer, true);
			<PendingVAD<T>>::insert(&claimer, false);

			// Emit an event.
			Self::deposit_event(Event::Approved(claimer));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	/* ----------------------------------------------------------------------------------------- */	

		// register vaccine type by only sysman
		// ex) Pfizer, Moderna ...
		#[pallet::weight(10_000)]
		pub fn register_vac_type(origin: OriginFor<T>) -> DispatchResult {

			let sysman = ensure_signed(origin)?;

			// only manufacture
			ensure!(Self::sys_man(sysman), Error::<T>::NotManufacture);

			// create vaccine type information
			let vac_type_id = VaccineTypeCount::<T>::get();
			let vac_type = VaccineTypeInfo {vac_type_id: Some(vac_type_id)};
			// TODO: Not safe math
			<VaccineTypeCount<T>>::put(vac_type_id + 1);

			// Update storage.
			<VaccineType<T>>::insert(&vac_type_id, vac_type);

			// Emit an event.
			Self::deposit_event(Event::RegisterVaccineType(vac_type_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// register vaccine information by only manufacture
		#[pallet::weight(10_000)]
		pub fn register_vac_info(origin: OriginFor<T>, vac_type_id: Option<u32>) -> DispatchResult {

			let manufacture = ensure_signed(origin)?;

			// only manufacture
			ensure!(Self::vm(&manufacture), Error::<T>::NotManufacture);
			// confirm exist vaccine type
			ensure!(<VaccineType<T>>::contains_key(vac_type_id.unwrap()), Error::<T>::NotRegisteredVaccineType);

			let vac_id = VaccineCount::<T>::get();

			match Vaccines::<T>::try_get(vac_type_id.unwrap()){

				Ok(_) => return Err(Error::<T>::VaccineIsRegistered)?,
				Err(_) => {
					let vac_info = VaccineInfo::<AccountIdOf<T>, BoundedVec<AccountIdOf<T>, T::MaxListSize>>{
						vac_id: Some(vac_id),
						manufacture_id: Some(manufacture.clone()),
						owner_id: Some(manufacture),
						buyer_id: None,
						vao_list: Default::default(),
						buy_confirm: false,
						vac_type_id,
						max_inoculations_number: 8,
						inoculation_count: 0,
						approved_time: None,
					};
					// Update storage.     
					//<VaccineCount<T>>::put(vac_id + 1);
					<VaccineCount<T>>::mutate(|count|{
						*count +=1;
					});
					<Vaccines<T>>::insert(&vac_id, vac_info);
				}

			};

			// if let Some(vac) = vaccinfo {
				
			// }


			// Emit an event.
			Self::deposit_event(Event::RegisterVaccine(vac_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// transfer vaccine by only manufacture and distributer
		#[pallet::weight(10_000)]
		pub fn transfer_vaccine(origin: OriginFor<T>, buyer_id: T::AccountId, vac_id: Option<u32>) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			// only manufacture or distributer
			ensure!(Self::vm(&sender) || Self::vad(&sender), Error::<T>::NotManufacture);

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(vac_id.unwrap()), Error::<T>::NotRegisteredVaccine);

			// confirm buyer is not me
			ensure!(sender != buyer_id, Error::<T>::BuyByYourself);

			// vaccine info のownerがsenderか確認
			let vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			let owner_id = vac_info.owner_id.unwrap();
			ensure!(owner_id == sender, Error::<T>::WrongVaccineOwner);
			// confirm vaccine not used
			let count = vac_info.inoculation_count;
			ensure!(count == 0, Error::<T>::VaccineAlreadyUsed);

			// 送る structとstorageの更新
			let mut new_vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			new_vac_info.buyer_id = Some(buyer_id);
			new_vac_info.buy_confirm = false;
			<Vaccines<T>>::insert(vac_id.unwrap(), new_vac_info);
			
			// Emit an event.
			Self::deposit_event(Event::TransferVaccine(vac_id.unwrap()));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// receive vaccine by only manufacture and distributer
		#[pallet::weight(10_000)]
		pub fn receive_vaccine(origin: OriginFor<T>, sender: T::AccountId, vac_id: Option<u32>) -> DispatchResult {
			let receiver = ensure_signed(origin)?;

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(vac_id.unwrap()), Error::<T>::NotRegisteredVaccine);

			// only manufacture or distributer
			ensure!(Self::vm(&receiver) || Self::vad(&receiver), Error::<T>::NotManufacture);

			// only specified receiver
			let vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			let buyer_id = vac_info.buyer_id.unwrap();
			ensure!(receiver == buyer_id, Error::<T>::NotVaccineBuyer);
			// confirm vaccine not used
			let count = vac_info.inoculation_count;
			ensure!(count == 0, Error::<T>::VaccineAlreadyUsed);
			// confirm vaccine will not transfer
			let confirmation = <Vaccines<T>>::get(vac_id.unwrap()).unwrap().buy_confirm;
			ensure!(!confirmation, Error::<T>::VaccineWillTransfer);
			// confirm correct vaccine owner
			let owner = <Vaccines<T>>::get(vac_id.unwrap()).unwrap().owner_id.unwrap();
			ensure!(owner == sender, Error::<T>::WrongVaccineOwner);

			// update struct and storage 
			let mut new_vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			new_vac_info.owner_id = Some(receiver.clone());
			match new_vac_info.buyer_id == new_vac_info.owner_id && new_vac_info.buy_confirm == false {
				true => new_vac_info.buy_confirm = true,
				false => return Err(Error::<T>::NotVaccineTransfered)?
			}
			<Vaccines<T>>::insert(vac_id.unwrap(), new_vac_info);

			// Emit an event.
			Self::deposit_event(Event::ReceiveVaccine(receiver, sender, vac_id.unwrap()));

			Ok(())
		}

		// approve vaccine by only approved organization
		#[pallet::weight(10_000)]
		pub fn approve_vaccine(origin: OriginFor<T>, vac_id: Option<u32>) -> DispatchResult {

			let organization = ensure_signed(origin)?;

			// only approved organization
			ensure!(Self::vao(&organization), Error::<T>::NotOrganization);
			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(vac_id.unwrap()), Error::<T>::NotRegisteredVaccine);
			//TODO: confirm dont double approve by one organization

			// confirm vaccine not used
			let vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			let count = vac_info.inoculation_count;
			ensure!(count == 0, Error::<T>::VaccineAlreadyUsed);

			// upddate struct(vao list)
			let mut new_vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			new_vac_info.vao_list.try_push(organization.clone());
			// Update storage.
			<Vaccines<T>>::insert(&vac_id.unwrap(), new_vac_info);

			// Emit an event.
			Self::deposit_event(Event::VaccineApproved(vac_id.unwrap(), organization));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// finally transfer vaccine to user
		#[pallet::weight(10_000)]
		pub fn transfer_get_vaccine_right(origin: OriginFor<T>, user_id: T::AccountId, vac_id: Option<u32>) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			// only manufacture or distributer
			ensure!(Self::vm(&sender) || Self::vad(&sender), Error::<T>::NotManufacture);

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(vac_id.unwrap()), Error::<T>::NotRegisteredVaccine);

			// confirm vaccine not duplicated used
			ensure!(!<UsedVaccine<T>>::get(&vac_id.unwrap(), &user_id), Error::<T>::VaccineAlreadyUsed);

			// confirm buyer is not me
			ensure!(sender != user_id, Error::<T>::BuyByYourself);

			// vaccine info のownerがsenderか確認
			let vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			let owner_id = vac_info.owner_id.unwrap();
			ensure!(owner_id == sender, Error::<T>::WrongVaccineOwner);

			// 送る structとstorageの更新
			let mut new_vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			new_vac_info.buyer_id = Some(user_id.clone());
			new_vac_info.buy_confirm = false;
			// confirm inoculation count dont reach max number
			ensure!(new_vac_info.inoculation_count < new_vac_info.max_inoculations_number, Error::<T>::ExceedMaxShotNumber);
			new_vac_info.inoculation_count += 1;
			<Vaccines<T>>::insert(vac_id.unwrap(), new_vac_info);

			// register vaccine is used
			<UsedVaccine<T>>::insert(&vac_id.unwrap(), user_id, true);
			
			// Emit an event.
			Self::deposit_event(Event::TransferVaccine(vac_id.unwrap()));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// user confirm vaccine
		#[pallet::weight(10_000)]
		pub fn confirm_vaccine(origin: OriginFor<T>, vac_owner: Option<T::AccountId>,  vac_id: Option<u32>) -> DispatchResult {

			let user = ensure_signed(origin)?;

			// confirm send_final_transfer is sended to me?
			ensure!(<UsedVaccine<T>>::get(&vac_id.unwrap(), &user), Error::<T>::NotSendFinalTransfer);

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(vac_id.unwrap()), Error::<T>::NotRegisteredVaccine);

			// confirm vaccine owner
			let owner = <Vaccines<T>>::get(vac_id.unwrap()).unwrap().owner_id;
			let confirmation = <Vaccines<T>>::get(vac_id.unwrap()).unwrap().buy_confirm;
			// only specified receiver
			let vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			let buyer_id = vac_info.buyer_id.unwrap();
			ensure!(user == buyer_id, Error::<T>::NotVaccineBuyer);
			// confirm vaccine will not transfer
			ensure!(!confirmation, Error::<T>::VaccineWillTransfer);
			// confirm vaccine correct owner
			ensure!(owner == vac_owner, Error::<T>::WrongVaccineOwner);

			// update struct and storage 
			let mut new_vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			new_vac_info.owner_id = Some(user.clone());
			match new_vac_info.buyer_id == new_vac_info.owner_id && new_vac_info.buy_confirm == false {
				true => new_vac_info.buy_confirm = true,
				false => return Err(Error::<T>::NotVaccineTransfered)?
			}
			<Vaccines<T>>::insert(vac_id.unwrap(), new_vac_info);

			// issuing vaccine passport
			Self::register_vac_pass(user.clone(), vac_id);

			// Emit an event.
			Self::deposit_event(Event::HadVaccination(vac_id.unwrap(), user));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
/*----------------------------------------------helper function ------------------------------------------------- */
	impl<T: Config> Pallet<T> {


		pub fn register_vac_pass(registrant: T::AccountId, vac_id: Option<u32>) -> DispatchResult {
			match VaccinePassports::<T>::try_get(&registrant){
				Ok(_) => {
				},
				Err(_) => {
					let vac_pass = PassportInfo::<AccountIdOf<T>, BoundedVec<u32, T::MaxListSize>>{
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
			passport.vac_list.try_push(vac_id.unwrap());
			passport.inoculation_count += 1;

			// Update storage.     
			<VaccinePassports<T>>::insert(&registrant, passport);


			Ok(())
		}
	}
}
