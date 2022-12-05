#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;
pub mod parser;
use types::{ProofStr, VkeyStr};
use parser::{parse_proof, parse_vkey};
use bellman_verifier::{prepare_verifying_key, verify_proof};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_std::str::from_utf8;
use bls12_381::Bls12;
use ff::PrimeField as Fr;
use scale_info::prelude::string::String;

pub trait VerifierPallet<AccountId> {
	fn verifier(who: AccountId, proof_a: Vec<u8>, proof_b: Vec<u8>, proof_c: Vec<u8>, input1: Vec<u8>, input2: Vec<u8>) -> DispatchResult;
}



#[frame_support::pallet]
pub mod pallet {
	pub use super::*;



	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proof_store)]
	pub type Pof<T: Config> = StorageMap<_, Blake2_128Concat, u8, ProofStr, OptionQuery>;


	#[pallet::storage]
	#[pallet::getter(fn vkey_store)]
	pub type Vkey<T: Config> = StorageMap<_, Blake2_128Concat, u8, VkeyStr, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProofStored(ProofStr, T::AccountId),
		VerificationKeyStore(VkeyStr, T::AccountId),
		VerificationPassed(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoProof,
		NoVerificationKey,
		VerificationFailed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn add_vkey(
			origin: OriginFor<T>,
			
			vk_alpha1: Vec<u8>,
			vk_beta_1: Vec<u8>,
			vk_beta_2: Vec<u8>,
			vk_gamma_2: Vec<u8>,
			vk_delta_1: Vec<u8>,
			vk_delta_2: Vec<u8>,
			vk_ic: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			let vkey = VkeyStr {
				alpha_1: vk_alpha1,
				beta_1: vk_beta_1,
				beta_2: vk_beta_2,
				gamma_2: vk_gamma_2,
				delta_1: vk_delta_1,
				delta_2: vk_delta_2,
				ic: vk_ic,
			};
	
			<Vkey<T>>::insert(1, &vkey);
			
			Self::deposit_event(Event::<T>::VerificationKeyStore(vkey, who));
			Ok(())
		}
	}
}



impl<T: Config> VerifierPallet<T::AccountId> for  Pallet<T> {
	fn verifier(
		who: T::AccountId,
		proof_a: Vec<u8>,
		proof_b: Vec<u8>,
		proof_c: Vec<u8>, 
		input1: Vec<u8>,
		input2: Vec<u8>
	) -> DispatchResult {
		
		let proof = ProofStr { pi_a: proof_a, pi_b: proof_b, pi_c: proof_c };
		<Pof<T>>::insert(1, &proof);
		Self::deposit_event(Event::<T>::ProofStored(proof, who.clone()));

		match <Pof<T>>::get(1) {
			None => return Err(Error::<T>::NoProof.into()),
			Some(pof) => {
				log::info!("{:?}", pof.pi_a);
				let proof = parse_proof::<Bls12>(pof.clone());
				log::info!("{:?}", proof.a);

				match <Vkey<T>>::get(1) {
					None => return Err(Error::<T>::NoVerificationKey.into()),
					Some(vkeystr) => {
						log::info!("{:?}",vkeystr.alpha_1);
						let vkey = parse_vkey::<Bls12>(vkeystr);
						log::info!("{:?}",vkey.clone().alpha_g1);

						let pvk =  prepare_verifying_key(&vkey);

						let input1_slice = input1.as_slice();
						let input1_string = input1_slice.iter().map(|&s| s as char).collect::<String>();
						let new_input1 = String::from_utf8(input1_slice.to_vec()).unwrap();

						let input2_slice = input2.as_slice();
						let input2_string = input2_slice.iter().map(|&s| s as char).collect::<String>();
						let new_input2 = String::from_utf8(input2_slice.to_vec()).unwrap();

	
						match verify_proof(&pvk, &proof, &[Fr::from_str_vartime(&new_input1).unwrap(), Fr::from_str_vartime(&new_input2).unwrap()]) {
							Ok(()) => Self::deposit_event(Event::<T>::VerificationPassed(who)),
							Err(e) => {
								log::info!("{:?}", e);
								()
							}
						}

					}
				}
			},
		}

		Ok(())
	}

}