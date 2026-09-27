import Relay.Confirmation

/-!
Optional ChaCha20-Poly1305-PSIV adapter contract, imported explicitly as `Relay.PSIV`.
No cryptographic implementation or state packing is introduced here. These are semantic types, not a C ABI.
-/
namespace Relay.PSIV

abbrev Bytes (n : Nat) := {bytes : ByteArray // bytes.size = n}
abbrev Key := Bytes 32
abbrev Nonce := Bytes 12
abbrev Tag := Bytes 16

def maxPlaintextBytes : Nat := 65536
def maxAssociatedDataBytes : Nat := 65536

structure Record where
  ciphertext : ByteArray
  tag : Tag

/-- The external nonce and AD are not embedded in the record. -/
def Record.encode (record : Record) : ByteArray := record.ciphertext ++ record.tag.val

inductive Error where
  | authenticationFailed
  | invalidLength
  | exhausted
  | backendFailure
  deriving DecidableEq, Repr

/-- Same session operations as the dependency's psiv_init/seal/open/clear.
`openRecord` must release plaintext only after authentication succeeds. Actual mutation,
erasure, constant-time behavior and cryptographic security are backend obligations. -/
structure API (m : Type → Type) where
  Context : Type
  «init» : Key → m Context
  «seal» : Context → Nonce → ByteArray → ByteArray → m (Except Error Record)
  openRecord : Context → Nonce → ByteArray → Record → m (Except Error ByteArray)
  clear : Context → m Unit

/-- Compatibility alias for integrations that import the optional PSIV module. -/
abbrev ConfirmationContext := Relay.ConfirmationContext

end Relay.PSIV
