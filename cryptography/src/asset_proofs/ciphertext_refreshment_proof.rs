//! The proof that 2 cipher texts encrypt the same value
//! under the same public key.
//! This proof is useful to prove the correctness of a
//! ciphertext refreshment method.
//! For more details see sections 3.6 and 5.3 of the
//! whitepaper.

use crate::{
    asset_proofs::{
        encryption_proofs::{
            AssetProofProver, AssetProofProverAwaitingChallenge, AssetProofVerifier, ZKPChallenge,
        },
        transcript::{TranscriptProtocol, UpdateTranscript},
        CipherText, ElgamalPublicKey, ElgamalSecretKey,
    },
    errors::{ErrorKind, Fallible},
};
use bulletproofs::PedersenGens;
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT, ristretto::RistrettoPoint, scalar::Scalar,
};
use merlin::{Transcript, TranscriptRng};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// The domain label for the ciphertext refreshment proof.
pub const CIPHERTEXT_REFRESHMENT_FINAL_RESPONSE_LABEL: &[u8] =
    b"PolymathCipherTextRefreshmentFinalResponse";
/// The domain label for the challenge.
pub const CIPHERTEXT_REFRESHMENT_PROOF_CHALLENGE_LABEL: &[u8] =
    b"PolymathCipherTextRefreshmentChallenge";

// ------------------------------------------------------------------------
// Proof of two ciphertext encrypting the same value under the same
// public key
// ------------------------------------------------------------------------

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Debug, Default)]
pub struct CipherTextRefreshmentFinalResponse(Scalar);

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Debug)]
pub struct CipherTextRefreshmentInitialMessage {
    a: RistrettoPoint,
    b: RistrettoPoint,
}

/// A default implementation used for testing.
impl Default for CipherTextRefreshmentInitialMessage {
    fn default() -> Self {
        CipherTextRefreshmentInitialMessage {
            a: RISTRETTO_BASEPOINT_POINT,
            b: RISTRETTO_BASEPOINT_POINT,
        }
    }
}

impl UpdateTranscript for CipherTextRefreshmentInitialMessage {
    fn update_transcript(&self, transcript: &mut Transcript) -> Fallible<()> {
        transcript.append_domain_separator(CIPHERTEXT_REFRESHMENT_PROOF_CHALLENGE_LABEL);
        transcript.append_validated_point(b"A", &self.a.compress())?;
        transcript.append_validated_point(b"B", &self.b.compress())?;
        Ok(())
    }
}

pub struct CipherTextRefreshmentProverAwaitingChallenge<'a> {
    /// The public key used for the elgamal encryption.
    secret_key: ElgamalSecretKey,

    /// The difference between the Y part of the two ciphertexts:
    /// Y = ciphertext1.y - ciphertext2.y
    y: RistrettoPoint,
    pc_gens: &'a PedersenGens,
}

impl<'a> CipherTextRefreshmentProverAwaitingChallenge<'a> {
    pub fn new(
        secret_key: ElgamalSecretKey,
        ciphertext1: CipherText,
        ciphertext2: CipherText,
        gens: &'a PedersenGens,
    ) -> Self {
        CipherTextRefreshmentProverAwaitingChallenge {
            secret_key: secret_key,
            y: ciphertext1.y - ciphertext2.y,
            pc_gens: gens,
        }
    }
}

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct CipherTextRefreshmentProver {
    /// The secret key.
    secret_key: ElgamalSecretKey,

    /// The randomness generated in the first round.
    u: Scalar,
}

impl<'a> AssetProofProverAwaitingChallenge for CipherTextRefreshmentProverAwaitingChallenge<'a> {
    type ZKInitialMessage = CipherTextRefreshmentInitialMessage;
    type ZKFinalResponse = CipherTextRefreshmentFinalResponse;
    type ZKProver = CipherTextRefreshmentProver;

    fn create_transcript_rng<T: RngCore + CryptoRng>(
        &self,
        rng: &mut T,
        transcript: &Transcript,
    ) -> TranscriptRng {
        transcript
            .build_rng()
            .rekey_with_witness_bytes(b"y", self.y.compress().as_bytes())
            .finalize(rng)
    }

    fn generate_initial_message(
        &self,
        rng: &mut TranscriptRng,
    ) -> (Self::ZKProver, Self::ZKInitialMessage) {
        let rand_commitment = Scalar::random(rng);

        let initial_message = CipherTextRefreshmentInitialMessage {
            a: rand_commitment * self.y,
            b: rand_commitment * self.pc_gens.B_blinding,
        };

        let prover = CipherTextRefreshmentProver {
            secret_key: self.secret_key.clone(),
            u: rand_commitment,
        };
        (prover, initial_message)
    }
}

impl AssetProofProver<CipherTextRefreshmentFinalResponse> for CipherTextRefreshmentProver {
    fn apply_challenge(&self, c: &ZKPChallenge) -> CipherTextRefreshmentFinalResponse {
        CipherTextRefreshmentFinalResponse(self.u + c.x() * self.secret_key.secret)
    }
}

pub struct CipherTextRefreshmentVerifier<'a> {
    /// The public key to which the `value` is encrypted.
    pub pub_key: ElgamalPublicKey,

    /// The difference between the X part of the two ciphertexts:
    /// X = ciphertext1.x - ciphertext2.x
    pub x: RistrettoPoint,

    /// The difference between the Y part of the two ciphertexts:
    /// Y = ciphertext1.y - ciphertext2.y
    pub y: RistrettoPoint,
    pub pc_gens: &'a PedersenGens,
}

impl<'a> CipherTextRefreshmentVerifier<'a> {
    pub fn new(
        pub_key: ElgamalPublicKey,
        ciphertext1: CipherText,
        ciphertext2: CipherText,
        gens: &'a PedersenGens,
    ) -> Self {
        CipherTextRefreshmentVerifier {
            pub_key: pub_key,
            x: ciphertext1.x - ciphertext2.x,
            y: ciphertext1.y - ciphertext2.y,
            pc_gens: gens,
        }
    }
}

impl<'a> AssetProofVerifier for CipherTextRefreshmentVerifier<'a> {
    type ZKInitialMessage = CipherTextRefreshmentInitialMessage;
    type ZKFinalResponse = CipherTextRefreshmentFinalResponse;

    fn verify(
        &self,
        challenge: &ZKPChallenge,
        initial_message: &Self::ZKInitialMessage,
        z: &Self::ZKFinalResponse,
    ) -> Fallible<()> {
        ensure!(
            z.0 * self.y == initial_message.a + challenge.x() * self.x,
            ErrorKind::CiphertextRefreshmentFinalResponseVerificationError { check: 1 }
        );
        ensure!(
            z.0 * self.pc_gens.B_blinding
                == initial_message.b + challenge.x() * self.pub_key.pub_key,
            ErrorKind::CiphertextRefreshmentFinalResponseVerificationError { check: 2 }
        );
        Ok(())
    }
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    extern crate wasm_bindgen_test;
    use super::*;
    use crate::asset_proofs::*;
    use bincode::{deserialize, serialize};
    use rand::{rngs::StdRng, SeedableRng};
    use wasm_bindgen_test::*;

    const SEED_1: [u8; 32] = [17u8; 32];
    const SEED_2: [u8; 32] = [19u8; 32];

    #[test]
    #[wasm_bindgen_test]
    fn test_ciphertext_refreshment_proof() {
        let gens = PedersenGens::default();
        let mut rng = StdRng::from_seed(SEED_1);
        let secret_value = Scalar::from(13u32);

        let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
        let elg_pub = elg_secret.get_public_key();
        let ciphertext1 = elg_pub.encrypt_value(secret_value.clone(), &mut rng);
        let ciphertext2 = elg_pub.encrypt_value(secret_value.clone(), &mut rng);

        let prover = CipherTextRefreshmentProverAwaitingChallenge::new(
            elg_secret,
            ciphertext1,
            ciphertext2,
            &gens,
        );
        let verifier = CipherTextRefreshmentVerifier::new(elg_pub, ciphertext1, ciphertext2, &gens);
        let mut transcript = Transcript::new(CIPHERTEXT_REFRESHMENT_FINAL_RESPONSE_LABEL);

        // Positive tests
        let mut transcript_rng = prover.create_transcript_rng(&mut rng, &transcript);
        let (prover, initial_message) = prover.generate_initial_message(&mut transcript_rng);
        initial_message.update_transcript(&mut transcript).unwrap();
        let challenge = transcript
            .scalar_challenge(CIPHERTEXT_REFRESHMENT_PROOF_CHALLENGE_LABEL)
            .unwrap();
        let final_response = prover.apply_challenge(&challenge);

        let result = verifier.verify(&challenge, &initial_message, &final_response);
        assert!(result.is_ok());

        // Negative tests
        let bad_initial_message = CipherTextRefreshmentInitialMessage::default();
        let result = verifier.verify(&challenge, &bad_initial_message, &final_response);
        assert_err!(
            result,
            ErrorKind::CiphertextRefreshmentFinalResponseVerificationError { check: 1 }
        );

        let bad_final_response = CipherTextRefreshmentFinalResponse(Scalar::default());
        assert_err!(
            verifier.verify(&challenge, &initial_message, &bad_final_response),
            ErrorKind::CiphertextRefreshmentFinalResponseVerificationError { check: 1 }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn verify_ciphertext_refreshment_method() {
        let mut rng = StdRng::from_seed(SEED_2);
        let rand_blind = Scalar::random(&mut rng);
        let w = CommitmentWitness::new(3u32.into(), rand_blind);
        let gens = PedersenGens::default();
        let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
        let elg_pub = elg_secret.get_public_key();
        let cipher = elg_pub.encrypt(&w);

        let new_rand_blind = Scalar::random(&mut rng);
        let new_cipher = cipher.refresh(&elg_secret, new_rand_blind).unwrap();

        let prover = CipherTextRefreshmentProverAwaitingChallenge::new(
            elg_secret, cipher, new_cipher, &gens,
        );
        let verifier = CipherTextRefreshmentVerifier::new(elg_pub, cipher, new_cipher, &gens);

        let (initial_message, final_response) =
            encryption_proofs::single_property_prover(prover, &mut rng).unwrap();

        assert!(encryption_proofs::single_property_verifier(
            &verifier,
            initial_message,
            final_response
        )
        .is_ok());
    }

    #[test]
    #[wasm_bindgen_test]
    fn serialize_deserialize_proof() {
        let mut rng = StdRng::from_seed(SEED_1);
        let secret_value = Scalar::from(13u32);
        let gens = PedersenGens::default();
        let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
        let elg_pub = elg_secret.get_public_key();
        let ciphertext1 = elg_pub.encrypt_value(secret_value.clone(), &mut rng);
        let ciphertext2 = elg_pub.encrypt_value(secret_value.clone(), &mut rng);

        let prover = CipherTextRefreshmentProverAwaitingChallenge::new(
            elg_secret,
            ciphertext1,
            ciphertext2,
            &gens,
        );
        let (initial_message0, final_response0) = encryption_proofs::single_property_prover::<
            StdRng,
            CipherTextRefreshmentProverAwaitingChallenge,
        >(prover, &mut rng)
        .unwrap();

        let initial_message_bytes: Vec<u8> = serialize(&initial_message0).unwrap();
        let final_response_bytes: Vec<u8> = serialize(&final_response0).unwrap();
        let recovered_initial_message: CipherTextRefreshmentInitialMessage =
            deserialize(&initial_message_bytes).unwrap();
        let recovered_final_response: CipherTextRefreshmentFinalResponse =
            deserialize(&final_response_bytes).unwrap();
        assert_eq!(recovered_initial_message, initial_message0);
        assert_eq!(recovered_final_response, final_response0);
    }
}
