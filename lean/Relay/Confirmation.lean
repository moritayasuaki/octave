/-!
Cipher-independent session context for confirmation integrations. These semantic fields
specify required bindings; they do not implement a canonical codec, KDF or authenticator.
-/
namespace Relay

/-- Public context that the eventual canonical encoding/KDF/confirmation adapter must bind.
The core's confirmation predicate is instantiated once per such immutable transcript. -/
structure ConfirmationContext where
  protocolVersion : ByteArray
  suiteId : ByteArray
  profileId : ByteArray
  sessionId : ByteArray
  initiator : ByteArray
  responder : ByteArray
  relayRoster : Fin 8 → ByteArray
  direction : ByteArray
  challenge : ByteArray

end Relay
