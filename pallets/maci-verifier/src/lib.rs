// This file is part of Webb.

// Copyright (C) 2021 Webb Technologies Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Verifier Module
//!
//! A simple module for abstracting over arbitrary zero-knowledge verifiers
//! for arbitrary zero-knowledge gadgets. This pallet should store verifying
//! keys and any other verification specific parameters for different backends
//! that we support in Webb's ecosystem of runtime modules.
//!
//! ## Overview
//!
//! The Verifier module provides functionality for zero-knowledge verifier
//! management including:
//!
//! * Setting parameters for zero-knowledge verifier
//! * Setting the maintainer of the parameters
//!
//! To use it in your runtime, you need to implement the verifier [`Config`].
//! Additionally, you will want to implement the verifier traits defined in the
//! webb_primitives::verifier module.
//!
//! The supported dispatchable functions are documented in the [`Call`] enum.
//!
//! ### Terminology
//!
//! ### Goals
//!
//! The verifier system in Webb is designed to make the following possible:
//!
//! * Define.
//!
//! ## Interface
//!
//! ## Related Modules
//!
//! * [`System`](../frame_system/index.html)
//! * [`Support`](../frame_support/index.html)

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_std::convert::TryInto;
use sp_std::prelude::*;

use frame_support::pallet_prelude::{ensure, DispatchError, DispatchResult};
use arkworks_verifier::verify;
use ark_bls12_381::Bls12_381;

pub trait VerifierPallet {
	fn verifier(public_inp_bytes: &[u8], proof: &[u8]) -> DispatchResult;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	}


	#[pallet::storage]
	#[pallet::getter(fn proof_store)]
	pub type Pof<T: Config> = StorageMap<_, Blake2_128Concat, u8, Vec<u8>, OptionQuery>;


	#[pallet::storage]
	#[pallet::getter(fn vkey_store)]
	pub type Vkey<T: Config> = StorageMap<_, Blake2_128Concat, u8, Vec<u8>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProofStored(Vec<u8>),
		VerificationKeyStore(Vec<u8>),
		VerificationPassed(),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoProof,
		NoVerificationKey,
		VerificationFailed,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10000)]
		pub fn add_vkey(
			origin: OriginFor<T>,
			vkey: Vec<u8>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			
	
			<Vkey<T>>::insert(1, vkey.clone());
			
			Self::deposit_event(Event::<T>::VerificationKeyStore(vkey));
			Ok(())
		}
	}
}

impl<T: Config> VerifierPallet for Pallet<T> {

	fn verifier(public_inp_bytes: &[u8], proof: &[u8]) -> DispatchResult {

		<Pof<T>>::insert(1, proof);
		Self::deposit_event(Event::<T>::ProofStored(proof.to_vec()));

		match <Pof<T>>::get(1) {
			None => return Err(Error::<T>::NoProof.into()),
			Some(pof) => {
				// log::info!("{:?}", pof);

				match <Vkey<T>>::get(1) {
					None => return Err(Error::<T>::NoVerificationKey.into()),
					Some(vkeystr) => {
						// log::info!("{:?}",vkeystr);

						match verify::<Bls12_381>(public_inp_bytes, proof, &vkeystr) {
							Ok(true) => Self::deposit_event(Event::<T>::VerificationPassed()),
							Ok(false) => Err(Error::<T>::VerificationFailed)?,
							Err(e) => {
								log::info!("{:?}", e);
								Err(Error::<T>::VerificationFailed)?
							},
						}
					}
				}
			}
		}
		Ok(())
	}
}