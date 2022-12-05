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

pub trait VerifierPallet<AccountId> {
	fn verifier(who: AccountId,input: Vec<u8>) -> DispatchResult;
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
	pub type Pof<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ProofStr, OptionQuery>;


	#[pallet::storage]
	#[pallet::getter(fn vkey_store)]
	pub type Vkey<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, VkeyStr, OptionQuery>;

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
		pub fn generate_proof_vkey(
			origin: OriginFor<T>,
			proof_a: Vec<u8>,
			proof_b: Vec<u8>,
			proof_c: Vec<u8>,
			vk_alpha1: Vec<u8>,
			vk_beta_1: Vec<u8>,
			vk_beta_2: Vec<u8>,
			vk_gamma_2: Vec<u8>,
			vk_delta_1: Vec<u8>,
			vk_delta_2: Vec<u8>,
			vk_ic: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let proof = ProofStr { pi_a: proof_a, pi_b: proof_b, pi_c: proof_c };
			let vkey = VkeyStr {
				alpha_1: vk_alpha1,
				beta_1: vk_beta_1,
				beta_2: vk_beta_2,
				gamma_2: vk_gamma_2,
				delta_1: vk_delta_1,
				delta_2: vk_delta_2,
				ic: vk_ic,
			};
			// alpha_1: "[12, 193, 248, 239, 239, 177, 151, 100, 241, 227, 122, 76, 200, 133, 113, 95, 18, 165, 127, 39, 156, 108, 206, 223, 114, 226, 94, 14, 232, 212, 192, 140, 103, 182, 189, 218, 245, 208, 78, 113, 252, 47, 31, 183, 220, 87, 157, 115, 25, 128, 236, 20, 60, 251, 227, 247, 156, 193, 82, 186, 67, 190, 7, 231, 46, 17, 184, 42, 21, 215, 41, 138, 127, 176, 228, 41, 177, 173, 168, 246, 50, 38, 73, 59, 218, 69, 170, 230, 244, 31, 134, 222, 194, 37, 15, 184]".as_bytes().to_vec(),
			// 	beta_1: "[9, 50, 19, 243, 120, 142, 43, 67, 131, 164, 112, 50, 91, 252, 244, 31, 147, 168, 107, 24, 254, 255, 46, 193, 77, 115, 115, 11, 13, 59, 25, 115, 78, 55, 36, 96, 194, 192, 210, 105, 17, 230, 227, 52, 167, 175, 175, 95, 12, 183, 156, 249, 203, 66, 7, 74, 66, 138, 90, 249, 182, 47, 149, 20, 168, 27, 3, 190, 174, 236, 79, 209, 54, 202, 183, 247, 37, 77, 250, 167, 117, 46, 0, 46, 243, 168, 11, 61, 52, 79, 48, 139, 25, 137, 196, 116]".as_bytes().to_vec(),
			// 	beta_2: "[9, 83, 82, 65, 190, 86, 171, 165, 158, 32, 31, 42, 134, 83, 156, 10, 204, 5, 191, 103, 159, 158, 44, 0, 202, 89, 220, 153, 183, 185, 189, 229, 237, 248, 245, 48, 154, 166, 94, 238, 18, 129, 252, 78, 161, 61, 143, 160, 7, 27, 240, 142, 50, 83, 83, 183, 101, 184, 33, 82, 235, 92, 55, 196, 197, 209, 119, 210, 68, 185, 253, 173, 55, 139, 21, 161, 244, 195, 238, 26, 193, 223, 83, 158, 241, 250, 11, 139, 236, 23, 170, 2, 150, 73, 84, 107, 10, 102, 225, 83, 212, 41, 193, 120, 224, 239, 110, 225, 200, 146, 216, 16, 149, 11, 254, 189, 60, 227, 60, 27, 114, 128, 240, 40, 0, 66, 190, 236, 30, 61, 184, 137, 139, 187, 223, 137, 2, 50, 87, 37, 2, 255, 171, 165, 15, 70, 95, 67, 68, 129, 196, 215, 171, 22, 161, 221, 174, 234, 56, 185, 152, 20, 58, 247, 225, 57, 63, 116, 177, 173, 139, 148, 144, 62, 210, 187, 38, 198, 143, 83, 51, 174, 194, 236, 236, 216, 44, 118, 165, 167, 24, 128]".as_bytes().to_vec(),
			// 	gamma_2: "[2, 213, 229, 78, 239, 89, 79, 117, 249, 39, 187, 0, 191, 221, 136, 230, 216, 159, 221, 204, 45, 194, 237, 191, 39, 231, 46, 234, 41, 106, 49, 139, 241, 60, 108, 39, 172, 107, 122, 206, 9, 124, 116, 137, 119, 143, 183, 119, 2, 48, 131, 90, 65, 182, 170, 0, 19, 118, 120, 36, 35, 221, 91, 215, 16, 202, 228, 95, 113, 181, 249, 105, 156, 43, 138, 175, 211, 150, 127, 18, 42, 39, 189, 59, 17, 152, 164, 43, 193, 202, 149, 160, 255, 193, 196, 163, 5, 237, 174, 165, 196, 33, 26, 21, 61, 250, 230, 117, 157, 21, 187, 147, 50, 90, 174, 236, 184, 106, 240, 71, 225, 227, 95, 111, 122, 60, 3, 138, 127, 165, 55, 223, 45, 162, 173, 234, 201, 81, 136, 174, 249, 250, 30, 14, 24, 20, 57, 137, 54, 154, 16, 58, 230, 191, 151, 112, 131, 210, 156, 250, 201, 72, 55, 238, 219, 27, 76, 0, 84, 53, 9, 119, 253, 188, 197, 131, 12, 228, 105, 117, 47, 136, 196, 215, 9, 187, 100, 228, 12, 24, 158, 52]".as_bytes().to_vec(),
			// 	delta_1: "[24, 203, 255, 84, 54, 156, 184, 151, 8, 58, 46, 170, 211, 212, 99, 21, 237, 98, 29, 228, 87, 105, 52, 209, 120, 254, 14, 41, 207, 161, 51, 23, 163, 227, 139, 244, 172, 159, 235, 102, 70, 225, 120, 29, 231, 95, 157, 58, 24, 195, 163, 67, 129, 194, 34, 192, 131, 202, 104, 23, 168, 255, 116, 43, 71, 33, 121, 7, 127, 241, 69, 20, 243, 201, 68, 207, 55, 35, 51, 178, 95, 59, 239, 90, 80, 250, 159, 6, 18, 202, 124, 255, 239, 162, 221, 1]".as_bytes().to_vec(),
			// 	delta_2: "[15, 35, 169, 170, 244, 171, 239, 106, 51, 241, 140, 62, 119, 60, 201, 249, 162, 194, 219, 177, 14, 109, 59, 150, 231, 225, 112, 24, 245, 18, 111, 121, 44, 233, 137, 65, 226, 107, 255, 201, 75, 186, 83, 212, 226, 38, 213, 130, 14, 40, 147, 239, 45, 101, 164, 45, 204, 186, 125, 197, 183, 36, 59, 142, 218, 95, 42, 146, 77, 123, 121, 159, 3, 252, 209, 254, 30, 208, 95, 245, 50, 224, 240, 44, 71, 125, 4, 122, 33, 62, 235, 141, 195, 153, 40, 251, 16, 139, 65, 165, 184, 127, 186, 62, 11, 160, 187, 149, 198, 35, 123, 249, 111, 56, 30, 11, 147, 16, 99, 202, 209, 203, 84, 186, 193, 168, 90, 33, 254, 57, 133, 207, 231, 6, 17, 230, 87, 68, 174, 37, 193, 152, 212, 248, 7, 239, 106, 49, 216, 235, 181, 68, 100, 182, 225, 175, 0, 215, 108, 207, 149, 233, 50, 53, 104, 23, 195, 118, 73, 113, 141, 232, 100, 74, 92, 210, 117, 82, 198, 241, 36, 245, 47, 252, 152, 137, 76, 204, 40, 204, 49, 27]".as_bytes().to_vec(),
			// 	ic: ["[14, 25, 159, 107, 40, 58, 198, 82, 217, 158, 135, 124, 54, 14, 65, 157, 14, 240, 34, 82, 219, 40, 11, 168, 70, 188, 13, 193, 93, 237, 243, 137, 191, 57, 56, 40, 96, 249, 122, 242, 129, 139, 81, 109, 182, 87, 216, 48, 25, 138, 137, 235, 217, 88, 181, 34, 87, 195, 193, 9, 12, 206, 25, 229, 63, 194, 42, 232, 88, 239, 2, 81, 129, 3, 147, 86, 33, 208, 248, 161, 2, 4, 108, 158, 245, 146, 164, 174, 18, 144, 153, 1, 205, 146, 157, 14]".as_bytes().to_vec(), "[5, 61, 211, 52, 28, 62, 226, 188, 213, 198, 251, 103, 8, 108, 115, 5, 91, 198, 27, 182, 83, 253, 71, 130, 238, 54, 171, 88, 215, 28, 162, 13, 62, 62, 36, 33, 39, 107, 141, 250, 222, 0, 186, 184, 213, 191, 252, 93, 17, 31, 18, 133, 189, 216, 155, 83, 29, 8, 4, 180, 46, 91, 92, 15, 213, 244, 37, 69, 94, 113, 161, 42, 209, 107, 56, 160, 228, 240, 139, 155, 158, 171, 159, 239, 120, 162, 215, 132, 223, 170, 64, 29, 241, 112, 168, 86]".as_bytes().to_vec()].to_vec(),
			<Pof<T>>::insert(who.clone(), &proof);
			<Vkey<T>>::insert(who.clone(),&vkey);
			Self::deposit_event(Event::<T>::ProofStored(proof, who.clone()));
			Self::deposit_event(Event::<T>::VerificationKeyStore(vkey, who));
			Ok(())
		}

	

	}
}



impl<T: Config> VerifierPallet<T::AccountId> for  Pallet<T> {
	fn verifier(who: T::AccountId, input: Vec<u8>) -> DispatchResult {
		

		match <Pof<T>>::get(&who) {
			None => return Err(Error::<T>::NoProof.into()),
			Some(pof) => {
				log::info!("{:?}", pof.pi_a);
				let proof = parse_proof::<Bls12>(pof.clone());
				log::info!("{:?}", proof.a);

				match <Vkey<T>>::get(&who) {
					None => return Err(Error::<T>::NoVerificationKey.into()),
					Some(vkeystr) => {
						log::info!("{:?}",vkeystr.alpha_1);
						let vkey = parse_vkey::<Bls12>(vkeystr);
						log::info!("{:?}",vkey.clone().alpha_g1);

						let pvk =  prepare_verifying_key(&vkey);

						let input_slice = input.as_slice();
						let input_str: &str = from_utf8(&input_slice).unwrap();

						
						match verify_proof(&pvk, &proof, &[Fr::from_str_vartime(input_str).unwrap()]) {
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