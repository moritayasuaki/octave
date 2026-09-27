import Relay.Algebra
import Mathlib.SetTheory.Cardinal.Finite

open Polynomial

namespace Relay
noncomputable section

variable {F : Type*} [Field F]

/-- Polynomials consistent with a fixed relay view and a fixed secret. -/
def ViewFiber (observed : Finset RelayId) (nodes values : RelayId → F) (secret : F) :=
  {p : F[X] // p.degree < 4 ∧ p.eval 0 = secret ∧
    ∀ i ∈ observed, p.eval (nodes i) = values i}

instance viewFiber_finite [Finite F] (observed : Finset RelayId)
    (nodes values : RelayId → F) (secret : F) :
    Finite (ViewFiber observed nodes values secret) := by
  let coefficients : ViewFiber observed nodes values secret → (Fin 4 → F) :=
    fun p i => p.val.coeff i.val
  apply Finite.of_injective coefficients
  intro p q h
  apply Subtype.ext
  apply Polynomial.ext
  intro n
  by_cases hn : n < 4
  · exact congrFun h ⟨n, hn⟩
  · have hp := (degree_lt_iff_coeff_zero p.val 4).1 p.property.1 n (by omega)
    have hq := (degree_lt_iff_coeff_zero q.val 4).1 q.property.1 n (by omega)
    rw [hp, hq]

/-- At at most three nonzero points there is a cubic mask equal to zero at every
observed point and one at the secret point. Distinctness is not needed for this lemma. -/
theorem exists_privacy_mask (observed : Finset RelayId) (nodes : RelayId → F)
    (bound : observed.card ≤ 3) (nonzero : ∀ i ∈ observed, nodes i ≠ 0) :
    ∃ mask : F[X], mask.degree < 4 ∧ mask.eval 0 = 1 ∧
      ∀ i ∈ observed, mask.eval (nodes i) = 0 := by
  classical
  let points := insert 0 (observed.image nodes)
  let mask := Lagrange.interpolate points id (fun x => if x = 0 then (1 : F) else 0)
  have hn : Set.InjOn (id : F → F) points := Function.injective_id.injOn
  have hcard : points.card ≤ 4 := by
    have h₁ := Finset.card_insert_le (0 : F) (observed.image nodes)
    have h₂ := Finset.card_image_le (s := observed) (f := nodes)
    dsimp [points]
    omega
  refine ⟨mask, ?_, ?_, ?_⟩
  · exact (Lagrange.degree_interpolate_lt _ hn).trans_le (by exact_mod_cast hcard)
  · simpa [mask] using Lagrange.eval_interpolate_at_node
      (fun x => if x = 0 then (1 : F) else 0) hn (show (0 : F) ∈ points by simp [points])
  · intro i hi
    have hm : nodes i ∈ points := by simp [points, Finset.mem_image_of_mem nodes hi]
    simpa [mask, nonzero i hi] using Lagrange.eval_interpolate_at_node
      (fun x => if x = 0 then (1 : F) else 0) hn hm

/-- An explicit translation bijection between the two secret-conditioned view fibers.
This is the combinatorial core of perfect privacy; it makes no cipher assumption. -/
def privacyEquiv (observed : Finset RelayId) (nodes values : RelayId → F)
    (s₀ s₁ : F) (mask : F[X]) (hd : mask.degree < 4) (hz : mask.eval 0 = 1)
    (hv : ∀ i ∈ observed, mask.eval (nodes i) = 0) :
    ViewFiber observed nodes values s₀ ≃ ViewFiber observed nodes values s₁ where
  toFun p := ⟨p.val + (s₁ - s₀) • mask, by
    refine ⟨(degree_add_le _ _).trans_lt
      (max_lt p.property.1 ((degree_smul_le _ _).trans_lt hd)), ?_, ?_⟩
    · simp [eval_add, eval_smul, hz, p.property.2.1]
    · intro i hi
      simp [eval_add, eval_smul, hv i hi, p.property.2.2 i hi]⟩
  invFun p := ⟨p.val + (s₀ - s₁) • mask, by
    refine ⟨(degree_add_le _ _).trans_lt
      (max_lt p.property.1 ((degree_smul_le _ _).trans_lt hd)), ?_, ?_⟩
    · simp [eval_add, eval_smul, hz, p.property.2.1]
    · intro i hi
      simp [eval_add, eval_smul, hv i hi, p.property.2.2 i hi]⟩
  left_inv p := by
    apply Subtype.ext
    dsimp
    rw [add_assoc, ← add_smul]
    simp
  right_inv p := by
    apply Subtype.ext
    dsimp
    rw [add_assoc, ← add_smul]
    simp

theorem privacy_threshold_structure (observed : Finset RelayId) (nodes values : RelayId → F)
    (bound : observed.card ≤ 3) (nonzero : ∀ i ∈ observed, nodes i ≠ 0) (s₀ s₁ : F) :
    Nonempty (ViewFiber observed nodes values s₀ ≃ ViewFiber observed nodes values s₁) := by
  obtain ⟨mask, hd, hz, hv⟩ := exists_privacy_mask observed nodes bound nonzero
  exact ⟨privacyEquiv observed nodes values s₀ s₁ mask hd hz hv⟩

theorem view_fiber_card_independent (observed : Finset RelayId) (nodes values : RelayId → F)
    (bound : observed.card ≤ 3) (nonzero : ∀ i ∈ observed, nodes i ≠ 0) (s₀ s₁ : F) :
    Nat.card (ViewFiber observed nodes values s₀) = Nat.card (ViewFiber observed nodes values s₁) := by
  obtain ⟨e⟩ := privacy_threshold_structure observed nodes values bound nonzero s₀ s₁
  exact Nat.card_congr e

end
end Relay
