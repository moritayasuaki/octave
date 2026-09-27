import Relay.Parameters
import Mathlib.LinearAlgebra.Lagrange
import Mathlib.Tactic.ComputeDegree

open Polynomial
open scoped BigOperators

namespace Relay

section Field
variable {F : Type*} [Field F]

/-- The three coefficients must be independent uniform field samples, including zero. -/
def shareScalar (secret : F) (coins : Fin 3 → F) (x : F) : F :=
  secret + coins 0 * x + coins 1 * x ^ 2 + coins 2 * x ^ 3

noncomputable def sharingPolynomial (secret : F) (coins : Fin 3 → F) : F[X] :=
  C secret + C (coins 0) * X + C (coins 1) * X ^ 2 + C (coins 2) * X ^ 3

theorem sharingPolynomial_eval (secret : F) (coins : Fin 3 → F) (x : F) :
    (sharingPolynomial secret coins).eval x = shareScalar secret coins x := by
  simp [sharingPolynomial, shareScalar]

theorem sharingPolynomial_zero (secret : F) (coins : Fin 3 → F) :
    (sharingPolynomial secret coins).eval 0 = secret := by
  simp [sharingPolynomial]

theorem sharingPolynomial_degree (secret : F) (coins : Fin 3 → F) :
    (sharingPolynomial secret coins).degree < 4 := by
  unfold sharingPolynomial
  compute_degree!

/-- Lagrange interpolation evaluated at zero, with no polynomial allocation. -/
def reconstructScalar (s : Finset RelayId) (nodes values : RelayId → F) : F :=
  ∑ i ∈ s, values i * ∏ j ∈ s.erase i, ((nodes i - nodes j)⁻¹ * (0 - nodes j))

theorem reconstructScalar_eq_interpolate (s : Finset RelayId) (nodes values : RelayId → F) :
    reconstructScalar s nodes values = (Lagrange.interpolate s nodes values).eval 0 := by
  simp [reconstructScalar, Lagrange.interpolate_apply, Lagrange.basis,
    Lagrange.basisDivisor, eval_finsetSum, eval_prod]

theorem four_points_unique (nodes : RelayId → F) (s : Finset RelayId)
    (hn : Set.InjOn nodes s) (hs : s.card = 4) (p q : F[X])
    (hp : p.degree < 4) (hq : q.degree < 4)
    (h : ∀ i ∈ s, p.eval (nodes i) = q.eval (nodes i)) : p = q := by
  have hp' : p.degree < (s.card : WithBot Nat) := by simpa [hs] using hp
  have hq' : q.degree < (s.card : WithBot Nat) := by simpa [hs] using hq
  exact (Lagrange.eq_interpolate_of_eval_eq (fun i => q.eval (nodes i)) hn hp' h).trans
    (Lagrange.eq_interpolate hn hq').symm

theorem reconstruct_polynomial (nodes values : RelayId → F) (s : Finset RelayId)
    (hn : Set.InjOn nodes s) (hs : s.card = 4) (p : F[X]) (hp : p.degree < 4)
    (hv : ∀ i ∈ s, values i = p.eval (nodes i)) :
    reconstructScalar s nodes values = p.eval 0 := by
  rw [reconstructScalar_eq_interpolate]
  have he := Lagrange.eq_interpolate_of_eval_eq values hn
    (show p.degree < (s.card : WithBot Nat) by simpa [hs] using hp)
    (fun i hi => (hv i hi).symm)
  rw [← he]

theorem shamir_reconstruction_correct (secret : F) (coins : Fin 3 → F)
    (nodes values : RelayId → F) (s : Finset RelayId)
    (hn : Set.InjOn nodes s) (hs : s.card = 4)
    (hv : ∀ i ∈ s, values i = shareScalar secret coins (nodes i)) :
    reconstructScalar s nodes values = secret := by
  rw [reconstruct_polynomial nodes values s hn hs (sharingPolynomial secret coins)
    (sharingPolynomial_degree secret coins)]
  · exact sharingPolynomial_zero secret coins
  · intro i hi
    rw [sharingPolynomial_eval]
    exact hv i hi

end Field
end Relay
