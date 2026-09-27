import Relay

open Relay

private def check (label : String) (ok : Bool) : IO Unit :=
  unless ok do throw (IO.userError s!"FAIL: {label}")

private def testRoot (seed : Nat) : RootBytes :=
  fun b => ⟨(seed + 17 * b.val) % 256, Nat.mod_lt _ (by decide)⟩

/-- Deterministic PUBLIC test data. This is not a random number generator. -/
private def testCoins (seed : Nat) : Randomness :=
  fun b a => (seed + 31 * b.val + 73 * a.val : Nat)

private def testSelection : IO Unit := do
  check "empty" (selectUnique ([] : List Nat) (fun _ => true) == none)
  check "no confirmation" (selectUnique [1, 2, 3] (fun _ => false) == none)
  check "duplicate values are one candidate" (selectUnique [7, 7, 7] (fun _ => true) == some 7)
  check "reject two distinct confirmations" (selectUnique [7, 7, 8] (fun _ => true) == none)
  check "scan beyond first success" (selectUnique [8, 7, 7, 9] (fun n => n == 7 || n == 9) == none)
  check "one confirmed value" (selectUnique [8, 7, 7, 9] (fun n => n == 7) == some 7)

private def testSharing : IO Unit := do
  check "70 subsets" (allFourSubsets.length == 70)
  check "no duplicate subsets" (decide allFourSubsets.Nodup)
  for seed in [0, 1, 42, 255, 256, 1024] do
    let root := testRoot seed
    let secret := encodeRoot root
    let coins := testCoins seed
    check "byte root roundtrip" (decide (decodeRoot secret = some root))
    for subset in allFourSubsets do
      check "4-subset" (subset.card == 4)
      check "every 4-share reconstruction" (decide (reconstruct subset (share secret coins) = secret))
    let received : Received := fun i => some (share secret coins i)
    check "all honest candidates agree" (decide ((candidates received).all (fun s => decide (s = secret))))
    check "duplicate reconstructions accepted" (decide (recover received (fun s => decide (s = secret)) = some secret))
    check "byte-root recovery" (decide (recoverRoot received (fun r => decide (r = root)) = some root))
  let zeroCoins : Randomness := fun _ _ => 0
  let secret := encodeRoot (testRoot 0)
  for subset in allFourSubsets do
    check "degree below three allowed" (decide (reconstruct subset (share secret zeroCoins) = secret))
  let invalid : Secret := fun _ => 256
  check "field 256 is not a byte" (decide (decodeRoot invalid = none))
  check "invalid roots never accepted at public boundary"
    (decide (recoverRoot (fun i => some (share invalid (testCoins 9) i)) (fun _ => true) = none))
  for subset in allFourSubsets do
    check "field 256 still reconstructs algebraically"
      (decide (reconstruct subset (share invalid (testCoins 9)) = invalid))
  let expected := [63, 112, 219, 157, 213, 160, 28, 104]
  let actual := (List.finRange 8).map (fun i => (shareScalar (42 : F257) ![17, 256, 5] (node i)).val)
  check "independent scalar known-answer vector" (actual == expected)

private def testAvailability : IO Nat := do
  let secret := encodeRoot (testRoot 83)
  let shares := share secret (testCoins 2026)
  for n in List.range 9 do
    let received : Received := fun i => if i.val < n then some (shares i) else none
    check "choose(m,4) enumerated" ((candidates received).length == n.choose 4)
    check "candidate upper bound" ((candidates received).length ≤ 70)
    if n < 4 then
      check "insufficient shares fail" (decide (recover received (fun _ => true) = none))
  let mut placements := 0
  let corruptSets := ((List.finRange 8).sublistsLen 3).map List.toFinset
  for corrupt in corruptSets do
    for offline in List.finRange 8 do
      if offline ∉ corrupt then
        let received : Received := fun i =>
          if i = offline then none
          else if i ∈ corrupt then some (fun b => shares i b + (i.val + b.val + 1 : Nat))
          else some (shares i)
        check "7 received labels give 35 subsets" ((candidates received).length == 35)
        check "honest candidate exists" (decide (secret ∈ candidates received))
        check "3 corrupt + 1 offline recovery"
          (decide (recover received (fun s => decide (s = secret)) = some secret))
        check "lost confirmation fails" (decide (recover received (fun _ => false) = none))
        placements := placements + 1
  check "all 280 placements covered" (placements == 280)
  return placements

def main : IO Unit := do
  testSelection
  testSharing
  let placements ← testAvailability
  IO.println s!"PASS: sharing/reconstruction, 70 subsets, byte validation, unique confirmation, and {placements} adversarial placements"
