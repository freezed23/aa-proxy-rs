# AARE WPP and initial AA negotiation repair (2026-11-02)

Stacks on source bba388447d1d362ab005ce651585919197475d45 / draft #14.
The owner explicitly requested this repair and an AAW1 ARMv7 build. No AARE change,
KHUD firmware/APK build, configuration change or device installation is included.

Observed private-log evidence (raw logs and phone/network identities are not committed):
- The latest bba3884 banner is present. In one session HU VERSION_REQUEST is received
  and the phone-side write returns; no phone version response is observed before a
  generic transfer-stall disconnect about ten seconds after proxy start.
- A later WPP retry receives ID8 while waiting for WifiStartResponse(ID7), then ID7
  while waiting for WifiConnectStatus(ID6). Previous code logged mismatched IDs but
  advanced the stage anyway and read the wrong payload as connection status.
- Phone logs show Wi-Fi/TCP connection followed by version timeout or socket/network
  loss. The dongle clock is unset, so absolute cross-device alignment is unverified.
  First-request delivery into the AARE frame parser remains UNKNOWN.

Implementation:
1. Normal WPP bootstrap replies to ID8 by echoing its payload with ID9, ignores ID9,
   and continues waiting for the expected response. Unexpected non-Ping IDs fail
   explicitly. A single 60s deadline per stage includes all Pings and writes.
2. Parse the actual ID6 status using a required proto2 int32 field instead of a
   guessed payload byte. Missing/malformed/nonzero status fails the bootstrap.
3. Shared per-session readiness bits arm streaming idle monitoring only once both
   MITM TLS directions complete. Pending negotiation has a separate 45s ceiling;
   existing 15s version receive deadlines and endpoint read limits still apply.
   Passthrough preserves immediate streaming monitoring. Idle window baselines
   reset at the transition; stop/reconnect actions stay responsive in both phases.
4. Packet transmission retries short writes, fails on zero progress and returns
   the full written byte count. The first MD version-write log now reports bytes,
   channel, flags and protocol version, without dumping arbitrary payloads.

Validation scope:
- Eight dependency-free regression tests exercise the actual production WPP
  classification and startup-monitor policy. Dockerfile.aaw1 runs them with host
  rustc before cross-compilation/export; a test failure fails the build.
- Tests cover Ping interleaving, wrong IDs, both TLS readiness bits, the initial
  ten-second false timeout, the bounded startup deadline, readiness transition,
  one-direction idle and active-stream baselines. These are policy tests, not
  a full application/Bluetooth/USB/TLS or end-to-end AARE test.
- ARM release compilation validates the integrated transport/protobuf callers.
  Actual device/Ping exchange/version reception/TLS/vehicle stability remain
  UNVERIFIED until a new device session is captured. No automatic deployment.
