#![cfg_attr(not(feature = "std"), no_std)]

use ark_crypto_primitives::{Error, SNARK};
use ark_ec::{PairingEngine};
use ark_ec::bls12;
use ark_ec::AffineCurve;
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use arkworks_native_gadgets::to_field_elements;
use sp_std::marker::PhantomData;
use ark_bls12_381::Bls12_381;
use ark_ff::PrimeField;
// pub struct ArkworksVerifierGroth16<E: PairingEngine>;

pub fn verify_groth16<E: PairingEngine>(
	vk: &VerifyingKey<E>,
	public_inputs: &[E::Fr],
	proof: &Proof<E>,
) -> Result<bool, Error> {
	let res = Groth16::<E>::verify(vk, public_inputs, proof)?;
	Ok(res)
}

pub fn verify<E: PairingEngine>(public_inp_bytes: &[u8], proof_bytes: &[u8], vk_bytes: &[u8]) -> Result<bool, Error> {
	let public_input_field_elts = to_field_elements::<E::Fr>(public_inp_bytes)?;
	let vk = VerifyingKey::<E>::deserialize(vk_bytes)?;
	let proof = Proof::<E>::deserialize(proof_bytes)?;
	let res = verify_groth16::<E>(&vk, &public_input_field_elts, &proof)?;
	Ok(res)
}

// use ark_bn254::Bn254;
// pub type ArkworksVerifierBn254 = ArkworksVerifierGroth16<Bn254>;

// use ark_bls12_381::Bls12_381;
// pub type ArkworksVerifierBls381 = ArkworksVerifierGroth16<Bls12_381>;