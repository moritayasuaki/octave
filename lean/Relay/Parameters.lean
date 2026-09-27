import Mathlib.Data.Finset.Powerset
import Mathlib.Data.Fintype.Fin
import Mathlib.Data.List.Sublists
import Mathlib.Tactic.NormNum

namespace Relay

def relayCount : Nat := 8
def threshold : Nat := 4
def maxCorrupt : Nat := 3
def maxOffline : Nat := 1
abbrev RelayId := Fin 8

theorem parameter_bounds : maxCorrupt < threshold ∧
    threshold + maxCorrupt + maxOffline = relayCount := by decide

theorem enoughHonest : relayCount - maxCorrupt - maxOffline ≥ threshold := by decide

/-- One slot per authenticated relay identity; corrupt/offline sets may overlap. -/
def honestAvailable (corrupt offline : Finset RelayId) : Finset RelayId :=
  Finset.univ \ (corrupt ∪ offline)

theorem four_honest_remain (corrupt offline : Finset RelayId)
    (hc : corrupt.card ≤ 3) (ho : offline.card ≤ 1) :
    4 ≤ (honestAvailable corrupt offline).card := by
  have hu := Finset.card_union_le corrupt offline
  have he := Finset.le_card_sdiff (corrupt ∪ offline) (Finset.univ : Finset RelayId)
  simp only [Finset.card_univ, Fintype.card_fin] at he
  unfold honestAvailable
  omega

/-- Deterministic enumeration, executable without choosing a quotient representative. -/
def allFourSubsets : List (Finset RelayId) :=
  ((List.finRange 8).sublistsLen 4).map List.toFinset

theorem allFourSubsets_length : allFourSubsets.length = 70 := by decide

theorem allFourSubsets_nodup : allFourSubsets.Nodup := by decide

theorem allFourSubsets_complete :
    allFourSubsets.toFinset = (Finset.univ : Finset RelayId).powersetCard 4 := by decide

theorem mem_allFourSubsets (s : Finset RelayId) :
    s ∈ allFourSubsets ↔ s.card = 4 := by
  rw [← List.mem_toFinset, allFourSubsets_complete, Finset.mem_powersetCard]
  simp

theorem exists_honest_four (corrupt offline : Finset RelayId)
    (hc : corrupt.card ≤ 3) (ho : offline.card ≤ 1) :
    ∃ s ∈ allFourSubsets, s ⊆ honestAvailable corrupt offline := by
  obtain ⟨s, hs, hcard⟩ := Finset.exists_subset_card_eq (four_honest_remain corrupt offline hc ho)
  exact ⟨s, (mem_allFourSubsets s).2 hcard, hs⟩

end Relay
