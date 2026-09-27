import Relay

-- This file prints transitive axiom dependencies of the principal claims.
-- None may depend on sorryAx or Lean.ofReduceBool (native_decide).
#print axioms Relay.parameter_bounds
#print axioms Relay.enoughHonest
#print axioms Relay.four_honest_remain
#print axioms Relay.allFourSubsets_length
#print axioms Relay.allFourSubsets_nodup
#print axioms Relay.allFourSubsets_complete
#print axioms Relay.exists_honest_four
#print axioms Relay.sharingPolynomial_degree
#print axioms Relay.four_points_unique
#print axioms Relay.reconstructScalar_eq_interpolate
#print axioms Relay.shamir_reconstruction_correct
#print axioms Relay.node_injective
#print axioms Relay.node_nonzero
#print axioms Relay.decode_encode_root
#print axioms Relay.reconstruct_share
#print axioms Relay.candidateSubsets_spec
#print axioms Relay.candidate_count_bound
#print axioms Relay.candidate_count_exact
#print axioms Relay.true_secret_present
#print axioms Relay.selectUnique_spec
#print axioms Relay.unique_selection_safe
#print axioms Relay.unique_selection_complete
#print axioms Relay.recover_safe
#print axioms Relay.recover_correct
#print axioms Relay.recoverRoot_safe
#print axioms Relay.recoverRoot_correct
#print axioms Relay.exists_privacy_mask
#print axioms Relay.viewFiber_finite
#print axioms Relay.privacy_threshold_structure
#print axioms Relay.view_fiber_card_independent
