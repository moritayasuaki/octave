import Relay.Algebra
import Relay.Selection
import Mathlib.Data.ZMod.Basic
import Mathlib.Algebra.Field.ZMod
import Mathlib.Tactic.NormNum.Prime

namespace Relay

abbrev F257 := ZMod 257
instance : Fact (Nat.Prime 257) := ⟨by norm_num⟩

/-- The label is zero-based internally and maps to field point label + 1. -/
def node (i : RelayId) : F257 := (i.val + 1 : Nat)

theorem node_injective : Function.Injective node := by
  intro i j h
  have hval := congrArg ZMod.val h
  simp only [node, ZMod.val_natCast] at hval
  have hi : i.val + 1 < 257 := by omega
  have hj : j.val + 1 < 257 := by omega
  rw [Nat.mod_eq_of_lt hi, Nat.mod_eq_of_lt hj] at hval
  apply Fin.ext
  omega

theorem node_nonzero (i : RelayId) : node i ≠ 0 := by
  intro h
  have hval := congrArg ZMod.val h
  simp only [node, ZMod.val_natCast, ZMod.val_zero] at hval
  have hi : i.val + 1 < 257 := by omega
  rw [Nat.mod_eq_of_lt hi] at hval
  omega

/-- A raw candidate can contain 256. Only canonical byte roots may reach confirmation. -/
abbrev Secret := Fin 32 → F257
abbrev RootBytes := Fin 32 → Fin 256
abbrev Randomness := Fin 32 → Fin 3 → F257
abbrev Shares := RelayId → Secret
abbrev Received := RelayId → Option Secret

def encodeRoot (root : RootBytes) : Secret := fun b => (root b).val

def decodeRoot (secret : Secret) : Option RootBytes :=
  if h : ∀ b, (secret b).val < 256 then
    some (fun b => ⟨(secret b).val, h b⟩)
  else none

theorem decode_encode_root (root : RootBytes) : decodeRoot (encodeRoot root) = some root := by
  have hv (b : Fin 32) : (encodeRoot root b).val = (root b).val := by
    exact ZMod.val_natCast_of_lt (by have := (root b).isLt; omega)
  have h : ∀ b, (encodeRoot root b).val < 256 := by
    intro b
    rw [hv]
    exact (root b).isLt
  simp only [decodeRoot, dif_pos h, Option.some.injEq]
  funext b
  apply Fin.ext
  exact hv b

def share (secret : Secret) (coins : Randomness) : Shares :=
  fun i b => shareScalar (secret b) (coins b) (node i)

def reconstruct (s : Finset RelayId) (shares : Shares) : Secret :=
  fun b => reconstructScalar s node (fun i => shares i b)

theorem reconstruct_share (secret : Secret) (coins : Randomness)
    (s : Finset RelayId) (hs : s.card = 4) :
    reconstruct s (share secret coins) = secret := by
  funext b
  exact shamir_reconstruction_correct (secret b) (coins b) node _ s
    node_injective.injOn hs (fun _ _ => rfl)

def available (received : Received) : Finset RelayId :=
  Finset.univ.filter (fun i => (received i).isSome)

def candidateSubsets (received : Received) : List (Finset RelayId) :=
  allFourSubsets.filter (fun s => decide (s ⊆ available received))

def candidates (received : Received) : List Secret :=
  (candidateSubsets received).map (fun s => reconstruct s (fun i => (received i).getD 0))

/-- `confirms` is fixed for one fully bound transcript. All candidate roots are checked. -/
def recover (received : Received) (confirms : Secret → Bool) : Option Secret :=
  selectUnique (candidates received) confirms

def canonicalConfirms (confirms : RootBytes → Bool) (raw : Secret) : Bool :=
  match decodeRoot raw with
  | none => false
  | some root => confirms root

/-- Public byte-root boundary: non-byte field values cannot reach the confirmation adapter. -/
def recoverRoot (received : Received) (confirms : RootBytes → Bool) : Option RootBytes :=
  (recover received (canonicalConfirms confirms)).bind decodeRoot

theorem candidateSubsets_spec (received : Received) (s : Finset RelayId) :
    s ∈ candidateSubsets received ↔ s.card = 4 ∧ s ⊆ available received := by
  simp [candidateSubsets, mem_allFourSubsets]

theorem candidate_count_bound (received : Received) : (candidates received).length ≤ 70 := by
  simp only [candidates, List.length_map]
  exact (List.length_filter_le _ _).trans_eq allFourSubsets_length

theorem candidate_count_exact (received : Received) :
    (candidates received).length = (available received).card.choose 4 := by
  have hn : (candidateSubsets received).Nodup := List.Nodup.filter _ allFourSubsets_nodup
  have he : (candidateSubsets received).toFinset = (available received).powersetCard 4 := by
    ext s
    simp only [List.mem_toFinset, candidateSubsets_spec, Finset.mem_powersetCard]
    exact and_comm
  simp only [candidates, List.length_map]
  rw [← List.toFinset_card_of_nodup hn, he, Finset.card_powersetCard]

/-- Honest means both available and unmodified at this session's labelled slot. -/
def HonestDelivery (received : Received) (secret : Secret) (coins : Randomness)
    (corrupt offline : Finset RelayId) : Prop :=
  ∀ i ∈ honestAvailable corrupt offline, received i = some (share secret coins i)

theorem true_secret_present (received : Received) (secret : Secret) (coins : Randomness)
    (corrupt offline : Finset RelayId) (hc : corrupt.card ≤ 3) (ho : offline.card ≤ 1)
    (delivery : HonestDelivery received secret coins corrupt offline) :
    secret ∈ candidates received := by
  obtain ⟨s, hmem, hsub⟩ := exists_honest_four corrupt offline hc ho
  have hs := (mem_allFourSubsets s).1 hmem
  have hsa : s ⊆ available received := by
    intro i hi
    simp [available, delivery i (hsub hi)]
  apply List.mem_map.mpr
  refine ⟨s, (candidateSubsets_spec received s).2 ⟨hs, hsa⟩, ?_⟩
  funext b
  apply shamir_reconstruction_correct (secret b) (coins b) node _ s node_injective.injOn hs
  intro i hi
  simp only [delivery i (hsub hi), Option.getD_some]
  rfl

theorem recover_safe (received : Received) (confirms : Secret → Bool) (real selected : Secret)
    (sound : ConfirmationSound (candidates received) confirms real)
    (accepted : recover received confirms = some selected) : selected = real :=
  unique_selection_safe _ _ _ _ sound accepted

theorem recover_correct (received : Received) (secret : Secret) (coins : Randomness)
    (corrupt offline : Finset RelayId) (hc : corrupt.card ≤ 3) (ho : offline.card ≤ 1)
    (delivery : HonestDelivery received secret coins corrupt offline)
    (confirms : Secret → Bool) (complete : confirms secret = true)
    (sound : ConfirmationSound (candidates received) confirms secret) :
    recover received confirms = some secret :=
  unique_selection_complete _ _ _
    (true_secret_present received secret coins corrupt offline hc ho delivery) complete sound

theorem recoverRoot_safe (received : Received) (confirms : RootBytes → Bool)
    (real selected : RootBytes)
    (sound : ConfirmationSound (candidates received) (canonicalConfirms confirms) (encodeRoot real))
    (accepted : recoverRoot received confirms = some selected) : selected = real := by
  obtain ⟨raw, hr, hd⟩ := Option.bind_eq_some_iff.mp accepted
  have he := recover_safe received (canonicalConfirms confirms) (encodeRoot real) raw sound hr
  rw [he, decode_encode_root] at hd
  exact (Option.some.inj hd).symm

theorem recoverRoot_correct (received : Received) (root : RootBytes) (coins : Randomness)
    (corrupt offline : Finset RelayId) (hc : corrupt.card ≤ 3) (ho : offline.card ≤ 1)
    (delivery : HonestDelivery received (encodeRoot root) coins corrupt offline)
    (confirms : RootBytes → Bool) (complete : confirms root = true)
    (sound : ConfirmationSound (candidates received) (canonicalConfirms confirms) (encodeRoot root)) :
    recoverRoot received confirms = some root := by
  have hconfirm : canonicalConfirms confirms (encodeRoot root) = true := by
    unfold canonicalConfirms
    rw [decode_encode_root]
    exact complete
  unfold recoverRoot
  rw [recover_correct received (encodeRoot root) coins corrupt offline hc ho delivery
    (canonicalConfirms confirms) hconfirm sound]
  exact decode_encode_root root

end Relay
