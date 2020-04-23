//! The `encryption_proofs` library contains API for generating
//! and verifying proofs of various properties of an encrypted
//! value proofs as part of the MERCAT
//! (Mediated, Encrypted, Reversible, SeCure Asset Transfers)
//! Project.
//!
//! For a full description of these proofs see section 5 of the
//! whitepaper. [todo: Add a link to the whitepaper.]
//!
//! Sigma protocols are a 3 round interactive protocols where
//! the prover convinces the verifier that a statement is true.
//!
//! Prover                         Dealer
//! - selects some random values
//!                       -->  [initial message]
//!                            - records the initial message
//!                            - deterministically calculates
//!                              a random challenge
//!           [challenge] <--
//! - generates a final response from the
//!   selected random values and
//!   the challenge
//!                       -->  [final response]
//!
//! Now given the `initial message` and the `final response` any
//! verifier can verify the prover's statement. Verifier uses the
//! transcript to generate the challenge:
//!
//! Verifier                       Dealer
//! - receives the [initial message, final response]
//!                       -->  [initial message]
//!                            - records the initial message
//!                            - deterministically calculates
//!                              a random challenge
//!           [challenge] <--
//! - verifies the final response
//!
//! The role of the Dealer can be eliminated if the challenge
//! could be generated deterministically but unpredictably from
//! the `initial message`. This technique is known as the
//! Fiat-Shamir huristic. We use Merlin transcripts as the
//! Dealer throughout this implementation.

use bulletproofs::PedersenGens;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use rand_core::{CryptoRng, RngCore};

use crate::asset_proofs::errors::AssetProofError;
use crate::asset_proofs::transcript::{TranscriptProtocol, UpdateTranscript};

/// The domain label for the encryption proofs.
pub const ENCRYPTION_PROOFS_LABEL: &[u8] = b"PolymathEncryptionProofs";
/// The domain label for the challenge.
pub const ENCRYPTION_PROOFS_CHALLENGE_LABEL: &[u8] = b"PolymathEncryptionProofsChallenge";

// ------------------------------------------------------------------------
// Sigma Protocol's Prover and Verifier Interfaces
// ------------------------------------------------------------------------

/// A scalar challenge.
pub struct ZKPChallenge {
    pub x: Scalar,
}

/// The interface for a 3-Sigma protocol.
/// Abstracting the prover and verifier roles.
///
/// Each proof needs to use the same `ZKInitialMessage` and `ZKFinalResponse` types
/// between the prover and the verifier.
/// Each `ZKInitialMessage` needs to implement the `UpdateTranscript` trait.
pub trait AssetProofProverAwaitingChallenge {
    type ZKInitialMessage: UpdateTranscript;
    type ZKFinalResponse;
    type ZKProver: AssetProofProver<Self::ZKFinalResponse>;

    /// First round of the Sigma protocol. Prover generates a initial message.
    ///
    /// # Inputs
    /// `pc_gens` The Pedersen Generators used for the Elgamal encryption.
    /// `rng`     An RNG.
    ///
    /// # Output
    /// A initial message.
    fn generate_initial_message<T: RngCore + CryptoRng>(
        &self,
        pc_gens: &PedersenGens,
        rng: &mut T,
    ) -> (Self::ZKProver, Self::ZKInitialMessage);
}

pub trait AssetProofProver<ZKFinalResponse> {
    /// Third round of the Sigma protocol. Prover receives a challenge and
    /// uses it to generate the final response.
    ///
    /// # Inputs
    /// `challenge` The scalar challenge, generated by the transcript.
    ///
    /// # Output
    /// A final response.
    fn apply_challenge(&self, challenge: &ZKPChallenge) -> ZKFinalResponse;
}

pub trait AssetProofVerifier {
    type ZKInitialMessage: UpdateTranscript;
    type ZKFinalResponse;

    /// Forth round of the Sigma protocol. Verifier receives the initial message
    /// and the final response, and verifies them.
    ///
    /// # Inputs
    /// `pc_gens`         The Pedersen Generators used for the Elgamal encryption.
    /// `challenge`       The scalar challenge, generated by the transcript.
    /// `initial_message` The initial message, generated by the Prover.
    /// `final_response`  The final response, generated by the Prover.
    ///
    /// # Output
    /// Ok on success, or an error on failure.
    fn verify(
        &self,
        pc_gens: &PedersenGens,
        challenge: &ZKPChallenge,
        initial_message: &Self::ZKInitialMessage,
        final_proof: &Self::ZKFinalResponse,
    ) -> Result<(), AssetProofError>;
}

// ------------------------------------------------------------------------
// Non-Interactive Zero Knowledge Proofs API
// ------------------------------------------------------------------------

/// The non-interactive implementation of the protocol for a single
/// encryption proof's prover role.
///
/// # Inputs
/// `prover` Any prover that implements the `AssetProofProver` trait.
/// `rng`    An RNG.
///
/// # Outputs
/// An initial message and a final response on success, or failure on an error.
pub fn single_property_prover<
    T: RngCore + CryptoRng,
    ProverAwaitingChallenge: AssetProofProverAwaitingChallenge,
>(
    prover_ac: ProverAwaitingChallenge,
    rng: &mut T,
) -> Result<
    (
        ProverAwaitingChallenge::ZKInitialMessage,
        ProverAwaitingChallenge::ZKFinalResponse,
    ),
    AssetProofError,
> {
    let (mut initial_messages, mut final_responses) =
        prove_multiple_encryption_properties(&[Box::new(prover_ac)], rng)?;
    Ok((initial_messages.remove(0), final_responses.remove(0)))
}

/// The non-interactive implementation of the protocol for a single
/// encryption proof's verifier role.
///
/// # Inputs
/// `verifier` Any verifier that implements the `AssetProofVerifier` trait.
/// `rng`      An RNG.
///
/// # Outputs
/// Ok on success, or failure on error.
pub fn single_property_verifier<Verifier: AssetProofVerifier>(
    verifier: &Verifier,
    initial_message: Verifier::ZKInitialMessage,
    final_response: Verifier::ZKFinalResponse,
) -> Result<(), AssetProofError> {
    verify_multiple_encryption_properties(&[verifier], (&[initial_message], &[final_response]))
}

/// The non-interactive implementation of the protocol for multiple provers
/// which use the same challenge. In this scenario the transcript combines all
/// the initial messages to generate a single challenge.
///
/// # Inputs
/// `provers` An array of provers that implement the
///           `AssetProofProverAwaitingChallenge` trait.
/// `rng`     An RNG.
///
/// # Outputs
/// An array of initial messages and proofs on success, or failure on error.
pub fn prove_multiple_encryption_properties<
    T: RngCore + CryptoRng,
    ProverAwaitingChallenge: AssetProofProverAwaitingChallenge,
>(
    provers: &[Box<ProverAwaitingChallenge>],
    rng: &mut T,
) -> Result<
    (
        Vec<ProverAwaitingChallenge::ZKInitialMessage>,
        Vec<ProverAwaitingChallenge::ZKFinalResponse>,
    ),
    AssetProofError,
> where {
    let mut transcript = Transcript::new(ENCRYPTION_PROOFS_LABEL);
    let gens = PedersenGens::default();

    let (provers_vec, initial_messages_vec): (Vec<_>, Vec<_>) = provers
        .iter()
        .map(|p| p.generate_initial_message(&gens, rng))
        .unzip();

    // Combine all the initial messages to create a single challenge.
    initial_messages_vec
        .iter()
        .map(|initial_message| initial_message.update_transcript(&mut transcript))
        .collect::<Result<(), _>>()?;

    let challenge = transcript.scalar_challenge(ENCRYPTION_PROOFS_CHALLENGE_LABEL);

    let final_responses: Vec<_> = provers_vec
        .into_iter()
        .map(|prover| prover.apply_challenge(&challenge))
        .collect::<Vec<_>>();

    Ok((initial_messages_vec, final_responses))
}

/// The non-interactive implementation of the protocol for multiple verifiers
/// which use the same challenge. In this scenario the transcript combines all
/// the initial messages to generate a single challenge.
///
/// # Inputs
/// `verifiers` An array of verifiers that implement the `AssetProofVerifier` trait.
/// `rng`       An RNG.
///
/// # Outputs
/// Ok on success, or failure on error.
pub fn verify_multiple_encryption_properties<Verifier: AssetProofVerifier>(
    verifiers: &[&Verifier],
    (initial_messages, final_responses): (
        &[Verifier::ZKInitialMessage],
        &[Verifier::ZKFinalResponse],
    ),
) -> Result<(), AssetProofError> {
    if initial_messages.len() != final_responses.len() || verifiers.len() != final_responses.len() {
        return Err(AssetProofError::VerificationError);
    }

    let mut transcript = Transcript::new(ENCRYPTION_PROOFS_LABEL);
    let gens = PedersenGens::default();

    // Combine all the initial messages to create a single challenge.
    initial_messages
        .iter()
        .map(|initial_message| initial_message.update_transcript(&mut transcript))
        .collect::<Result<(), _>>()?;

    let challenge = transcript.scalar_challenge(ENCRYPTION_PROOFS_CHALLENGE_LABEL);
    for i in 0..verifiers.len() {
        verifiers[i].verify(&gens, &challenge, &initial_messages[i], &final_responses[i])?;
    }

    Ok(())
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    extern crate wasm_bindgen_test;
    use super::*;
    use crate::asset_proofs::correctness_proof::{
        CorrectnessInitialMessage, CorrectnessProverAwaitingChallenge, CorrectnessVerifier,
    };
    use crate::asset_proofs::{CommitmentWitness, ElgamalSecretKey};
    use rand::{rngs::StdRng, SeedableRng};
    use rand_core::{CryptoRng, RngCore};
    use wasm_bindgen_test::*;

    const SEED_1: [u8; 32] = [42u8; 32];
    const SEED_2: [u8; 32] = [7u8; 32];

    fn create_correctness_proof_objects_helper<T: RngCore + CryptoRng>(
        plain_text: u32,
        rng: &mut T,
    ) -> (CorrectnessProverAwaitingChallenge, CorrectnessVerifier) {
        let rand_blind = Scalar::random(rng);
        let w = CommitmentWitness::new(plain_text, rand_blind).unwrap();

        let elg_secret = ElgamalSecretKey::new(Scalar::random(rng));
        let elg_pub = elg_secret.get_public_key();
        let cipher = elg_pub.encrypt(&w);

        let prover = CorrectnessProverAwaitingChallenge::new(&elg_pub, &w);
        let verifier = CorrectnessVerifier::new(&plain_text, &elg_pub, &cipher);

        (prover, verifier)
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_single_proof() {
        let mut rng = StdRng::from_seed(SEED_1);
        let secret_value = 42u32;

        let (prover, verifier) = create_correctness_proof_objects_helper(secret_value, &mut rng);
        let (initial_message, final_response) =
            single_property_prover::<StdRng, CorrectnessProverAwaitingChallenge>(prover, &mut rng)
                .unwrap();

        // Positive test
        assert_eq!(
            single_property_verifier(&verifier, initial_message, final_response),
            Ok(())
        );

        // Negative tests
        let bad_initial_message = CorrectnessInitialMessage::default();
        assert_eq!(
            single_property_verifier(&verifier, bad_initial_message, final_response),
            Err(AssetProofError::CorrectnessFinalResponseVerificationError {
                str: String::from("First Check")
            })
        );

        let bad_final_response = Scalar::one();
        assert_eq!(
            single_property_verifier(&verifier, initial_message, bad_final_response),
            Err(AssetProofError::CorrectnessFinalResponseVerificationError {
                str: String::from("First Check")
            })
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn multiple_proofs() {
        let mut rng = StdRng::from_seed(SEED_2);
        let secret_value1 = 6u32;
        let secret_value2 = 7u32;

        let (prover1, verifier1) = create_correctness_proof_objects_helper(secret_value1, &mut rng);
        let (prover2, verifier2) = create_correctness_proof_objects_helper(secret_value2, &mut rng);

        let provers_vec = [Box::new(prover1), Box::new(prover2)];

        let (initial_messages, final_responses) =
            prove_multiple_encryption_properties(&provers_vec, &mut rng).unwrap();

        let verifiers_vec = vec![&verifier1, &verifier2];
        assert_eq!(
            verify_multiple_encryption_properties(
                &verifiers_vec,
                (&initial_messages, &final_responses)
            ),
            Ok(())
        );

        // Negative tests
        let mut bad_initial_messages = initial_messages.clone();
        bad_initial_messages.remove(1);
        // Missmatched initial messages and final responses sizes
        assert_eq!(
            verify_multiple_encryption_properties(
                &verifiers_vec,
                (&bad_initial_messages, &final_responses)
            ),
            Err(AssetProofError::VerificationError)
        );

        // Corrupted initial message
        bad_initial_messages.push(CorrectnessInitialMessage::default());
        assert_eq!(
            verify_multiple_encryption_properties(
                &verifiers_vec,
                (&bad_initial_messages, &final_responses)
            ),
            Err(AssetProofError::CorrectnessFinalResponseVerificationError {
                str: String::from("First Check")
            })
        );

        // Corrupted final responses
        let mut bad_final_responses = final_responses.clone();
        bad_final_responses.remove(1);
        bad_final_responses.push(Scalar::default());
        assert_eq!(
            verify_multiple_encryption_properties(
                &verifiers_vec,
                (&initial_messages, &bad_final_responses)
            ),
            Err(AssetProofError::CorrectnessFinalResponseVerificationError {
                str: String::from("First Check")
            })
        );
    }
}
