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
use frame_support::{pallet_prelude::{*, ValueQuery}, traits::{Currency, tokens::ExistenceRequirement}};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use scale_info::TypeInfo;
	use serde::{Deserialize, Serialize};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MaxListSize: Get<u32>;
		type Currency: Currency<Self::AccountId>;
	}

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type VaccineTypeIndex = u32;
	pub type VaccineIndex = u32;

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct VaccineTypeInfo {
	    pub vac_type_id: Option<u32>,
		pub vac_value: Option<u32>,
		// TODO: add metadata or hash of metadata
		// pub metadata: Option<Vec<u8>>,
	}

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct VaccineInfo<Account, BoundedAccountList> {
	    pub vac_id: Option<u32>,
		pub manufacture_id: Option<Account>,
		pub owner_list: BoundedAccountList,
		pub vao_list: BoundedAccountList,
		// true -> buy, false -> not buy
		pub buy_confirm: bool,
		pub vac_type_id: Option<u32>,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


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

	#[pallet::storage]
	#[pallet::getter(fn vaccines)]
	pub type Vaccines<T: Config> = StorageMap<_, Blake2_128Concat, u32, VaccineInfo<AccountIdOf<T>, BoundedVec<AccountIdOf<T>, T::MaxListSize>>, OptionQuery>;

	#[pallet::type_value]
	pub fn InitialVaccineTypeCount<T: Config>() -> u32 { 1u32 }

	#[pallet::storage]
	#[pallet::getter(fn vaccine_type_count)]
	pub type VaccineTypeCount<T: Config> = StorageValue<_, VaccineTypeIndex, ValueQuery, InitialVaccineTypeCount<T>>;

	#[pallet::type_value]
	pub fn InitialVaccineCount<T: Config>() -> u32 { 1u32 }

	#[pallet::storage]
	#[pallet::getter(fn vaccine_count)]
	pub type VaccineCount<T: Config> = StorageValue<_, VaccineIndex, ValueQuery, InitialVaccineCount<T>>;

	#[pallet::storage]
	#[pallet::getter(fn want_to_buy)]
	// TODO: make vaccine list or manage many vaccines in box
	pub type WantToBuy<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, VaccineIndex, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn buyable_vaccine)]
	// TODO: make vaccine list or manage many vaccines in box
	pub type BuyableVaccine<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, VaccineIndex, OptionQuery>;

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
		RegisterVaccineType(u32, u32),
		RegisterVaccine(u32),
		RequestVaccine(u32),
		SellVaccine(u32),
		BuyVaccine(T::AccountId, T::AccountId, u32, BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyClaimed,
		AlreadyApproved,
		NotClaimed,
		NotSysMan,
		NotManufacture,
		NotRegisteredVaccineType,
		NotRegisteredVaccine,
		NotWantToBuy,
		NotApproved,
		NotEnoughBalance,
		VaccineIsRegistered,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

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

		#[pallet::weight(10_000)]
		pub fn register_vac_type(origin: OriginFor<T>, value: Option<u32>) -> DispatchResult {

			let manufacture = ensure_signed(origin)?;

			// only manufacture
			ensure!(Self::vm(manufacture), Error::<T>::NotManufacture);

			// create vaccine type information
			let vac_type_id = VaccineTypeCount::<T>::get();
			let vac_type = VaccineTypeInfo {vac_type_id: Some(vac_type_id), vac_value: value};
			// TODO: Not safe math
			<VaccineTypeCount<T>>::put(vac_type_id + 1);

			// Update storage.
			<VaccineType<T>>::insert(&vac_type_id, vac_type);

			// Emit an event.
			Self::deposit_event(Event::RegisterVaccineType(vac_type_id, value.unwrap()));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn register_vac_info(origin: OriginFor<T>, vac_type_id: Option<u32>) -> DispatchResult {

			let manufacture = ensure_signed(origin)?;

			// only manufacture
			ensure!(Self::vm(&manufacture), Error::<T>::NotManufacture);
			// confirm exist vaccine
			ensure!(<VaccineType<T>>::contains_key(vac_type_id.unwrap()), Error::<T>::NotRegisteredVaccineType);

			let vac_id = VaccineCount::<T>::get();

			match Vaccines::<T>::try_get(vac_type_id.unwrap()){

				Ok(_) => return Err(Error::<T>::VaccineIsRegistered)?,
				Err(_) => {
					let vac_info = VaccineInfo::<AccountIdOf<T>, BoundedVec<AccountIdOf<T>, T::MaxListSize>>{
						vac_id: Some(vac_id),
						manufacture_id: Some(manufacture.clone()),
						owner_list: Default::default(),
						vao_list: Default::default(),
						buy_confirm: false,
						vac_type_id,
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

		#[pallet::weight(10_000)]
		pub fn request_buy(origin: OriginFor<T>, vac_id: Option<u32>, target_id: T::AccountId) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// only manufacture
			ensure!(Self::vm(target_id), Error::<T>::NotManufacture);
			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(vac_id.unwrap()), Error::<T>::NotRegisteredVaccine);

			// register storage
			<WantToBuy<T>>::insert(buyer, vac_id.unwrap());

			// Emit an event.
			Self::deposit_event(Event::RequestVaccine(vac_id.unwrap()));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn sell_vaccine(origin: OriginFor<T>, vac_id: Option<u32>, buyer_id: T::AccountId) -> DispatchResult {

			let seller = ensure_signed(origin)?;

			// only manufacture
			ensure!(Self::vm(seller), Error::<T>::NotManufacture);

			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(vac_id.unwrap()), Error::<T>::NotRegisteredVaccine);

			// 買いたい人に売る。買う人はそれを承認して買う
			// 買いたいか確認
			ensure!(Self::want_to_buy(&buyer_id) == vac_id, Error::<T>::NotWantToBuy);
			// 売る関数（ストレージの更新）
			// Update storage.
			<BuyableVaccine<T>>::insert(&buyer_id, vac_id.unwrap());
			<WantToBuy<T>>::remove(&buyer_id);

			// Emit an event.
			Self::deposit_event(Event::SellVaccine(vac_id.unwrap()));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn buy_vaccine(origin: OriginFor<T>, vac_id: Option<u32>, seller: T::AccountId) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// only approved buyer
			ensure!(Self::buyable_vaccine(&buyer) == vac_id, Error::<T>::NotApproved);
			// confirm exist vaccine
			ensure!(<Vaccines<T>>::contains_key(vac_id.unwrap()), Error::<T>::NotRegisteredVaccine);
			// TODO: cofirm seller has buyable vac_id vaccine

			// register storage
			<BuyableVaccine<T>>::remove(&buyer);
			// update vaccine info
			let vac_info = <Vaccines<T>>::get(vac_id.unwrap()).unwrap();
			let type_id = vac_info.vac_type_id.unwrap();
			let bid_price = <VaccineType<T>>::get(type_id).unwrap().vac_value.unwrap();

			let mut new_vac_info = vac_info.clone();
			new_vac_info.owner_list.try_push(buyer.clone());
			<Vaccines<T>>::insert(vac_id.unwrap(), new_vac_info);
			// pay a fee
			// TODO: configure fee of vaccine
			// Check the buyer has enough free balance
			ensure!(T::Currency::free_balance(&buyer) >= bid_price.into(), <Error<T>>::NotEnoughBalance);
			T::Currency::transfer(&buyer, &seller, bid_price.into(), ExistenceRequirement::KeepAlive)?;

			// Emit an event.
			Self::deposit_event(Event::BuyVaccine(buyer, seller, vac_id.unwrap(), bid_price.into()));

			Ok(())
		}
	}
/*----------------------------------------------helper function ------------------------------------------------- */
	impl<T: Config> Pallet<T> {

		pub fn transfer_vaccine() {

		}
	}
}
