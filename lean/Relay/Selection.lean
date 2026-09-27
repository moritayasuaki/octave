import Relay.Parameters

namespace Relay

/-- Every occurrence of the same confirmed value is one candidate. The complete list is
checked before returning: two different confirmed values fail closed. -/
def selectUnique {α : Type*} [DecidableEq α] (candidates : List α) (confirms : α → Bool) : Option α :=
  match candidates.filter confirms with
  | [] => none
  | first :: rest => if ∀ x ∈ rest, x = first then some first else none

/-- Abstract per-transcript authentication assumption, deliberately not an AEAD theorem. -/
def ConfirmationSound {α : Type*} (candidates : List α) (confirms : α → Bool) (real : α) : Prop :=
  ∀ candidate ∈ candidates, confirms candidate = true → candidate = real

theorem selectUnique_spec {α : Type*} [DecidableEq α]
    (candidates : List α) (confirms : α → Bool) (selected : α) :
    selectUnique candidates confirms = some selected ↔
      selected ∈ candidates ∧ confirms selected = true ∧
      ∀ x ∈ candidates, confirms x = true → x = selected := by
  unfold selectUnique
  generalize he : candidates.filter confirms = accepted
  have hm : ∀ x, x ∈ accepted ↔ x ∈ candidates ∧ confirms x = true := by
    intro x
    rw [← he, List.mem_filter]
  cases accepted with
  | nil => simp_all
  | cons a rest =>
    by_cases hu : ∀ x ∈ rest, x = a
    · simp only [if_pos hu, Option.some.injEq]
      constructor
      · intro h
        subst selected
        obtain ⟨ha, hok⟩ := (hm a).1 (by simp)
        refine ⟨ha, hok, ?_⟩
        intro x hx hconf
        have := (hm x).2 ⟨hx, hconf⟩
        simp only [List.mem_cons] at this
        exact this.elim id (hu x)
      · rintro ⟨_, _, huniq⟩
        obtain ⟨ha, hok⟩ := (hm a).1 (by simp)
        exact huniq a ha hok
    · simp only [if_neg hu, reduceCtorEq, false_iff]
      rintro ⟨hs, hc, huniq⟩
      apply hu
      intro x hx
      obtain ⟨ha, hca⟩ := (hm a).1 (by simp)
      obtain ⟨hxm, hcx⟩ := (hm x).1 (by simp [hx])
      exact (huniq x hxm hcx).trans (huniq a ha hca).symm

theorem unique_selection_safe {α : Type*} [DecidableEq α]
    (candidates : List α) (confirms : α → Bool) (real selected : α)
    (sound : ConfirmationSound candidates confirms real)
    (accepted : selectUnique candidates confirms = some selected) : selected = real := by
  obtain ⟨hm, hc, _⟩ := (selectUnique_spec candidates confirms selected).1 accepted
  exact sound selected hm hc

theorem unique_selection_complete {α : Type*} [DecidableEq α]
    (candidates : List α) (confirms : α → Bool) (real : α)
    (present : real ∈ candidates) (complete : confirms real = true)
    (sound : ConfirmationSound candidates confirms real) :
    selectUnique candidates confirms = some real :=
  (selectUnique_spec candidates confirms real).2 ⟨present, complete, sound⟩

end Relay
