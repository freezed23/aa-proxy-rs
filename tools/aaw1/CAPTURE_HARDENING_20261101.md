# AAW1 diagnostic hardening after SDR repair — 2026-11-01

Scope: owner existing CS317 independent bench. Stacked on SDR PR #11 at
`dda510223125bfbd721f91bbc050dd6352c25bd9`, which itself stacks on #7.
Existing #11 build run41225353685 remains a preserved comparison/rollback asset,
not a completed log-privacy/capture fix. Its compile success was genuine; a
vehicle fault from that binary has not been observed in this work.

## Changes addressing the independent review

1. Web setting updates (ordinary, custom WASM and bulk WASM) log the key only.
   Custom Wi-Fi/nmcli/wpa_cli/wpactrl/shell failure summaries omit arguments,
   command bodies and child output. Actual configuration values/commands sent
   to the phone/network are unchanged. #11's ordinary credential-log and raw
   WifiInfoResponse suppression is retained. Known credential-bearing AA
   Bluetooth/Wi-Fi service payloads are also omitted from packet diagnostics.
2. Full-frame diagnostics validate declared/final lengths, flags, nonempty
   continuations, message size **256 KiB**, aggregate buffered payload **1 MiB**,
   **32 streams**, **128 fragments**, and **3s** age. Settings changes/disabling
   logging clear partial buffers; stale buffers expire on the next observed
   packet. Idle retention is still bounded. This is a passive logger, not a
   change to wire forwarding or support for editing fragmented SDRs.
3. Hex output has a hard **16 KiB** byte cap, including a request of 0/unlimited.
   Generic protobuf pretty input is capped at 16 KiB; formatted text at 4 KiB
   (plus labels/indentation). Legacy 0x8004 PNG is summarized by length/SHA256
   while turn number/angle remain visible. Original message/image bytes are not
   modified. Field-number ordering is not assumed for decoding.
4. Only diagnostic settings refresh on subsequent packets after web saves.
   The unchanged snapshot is compared without per-packet string allocation;
   protocol/SDR/routing options remain fixed for the active connection.
   Earlier version/SDR messages cannot be recovered retroactively: prepare the
   control profile before a new connection. Editing a file alone is not a live
   web save or a restart. Initial TLS/version negotiation uses startup settings.
5. Decrypted ingress packet diagnostics and optional files occur **before**
   WASM/SDR hooks. DecryptedOutput and HTTP SDR JSON remain the modified view.
   Active SDR options can still take effect once #11 parses the HU successfully;
   they are intentionally not silently reset. Review developer_mode, profiles,
   display injection, motion/speed/sink/vendor changes and scripts before trials.
6. Example profiles include focus, refusal, channel and service lifecycle.
   NAV filtering uses decoded service kinds, so media ACKs are excluded without
   guessing ch9 or a permanent channel exclusion list. If SDR decoding fails,
   stop and inspect control capture instead of inferring absent navigation.
   Sensor/GPS requests/batches are a separate mapped-channel test.
7. Opaque metadata profile identity is hashed separately from readable fallback
   text. FF and literal `non-UTF8[ff]` no longer select the same profile. Existing
   all-valid UTF-8 profile IDs are retained; invalid-metadata profiles may obtain
   a new ID and need the owner to review their overrides.

## Optional private binary capture

`pkt_debug_capture_dir` defaults empty/off. Explicitly set an absolute private
directory to capture original decrypted SDR (control 0006) and mapped NAV
8001..8008. Files contain **message payload including its 2B ID**, not the AA
transport header. File names identify direction/channel/message; success logs
contain byte length and SHA256, not captured contents.

Queue: **4 messages**, try-send before copying; full queue skips a file, never
waits for disk in the forwarding task. Per-process budget: **128 files / 32 MiB**,
each at most 256 KiB. A background writer creates a 0700 directory and 0600
create-new files; shared or final-symlink directories are rejected. Existing
parent directories are not permission-modified. Queue/disk failure does not
stop AA. Logs/pretty hashing still have processing cost; real-CS317 timing is
unverified. Pending files may be lost on process shutdown; verify success hashes.

No device settings/directory/installation are changed by building this source.
Captured packets can contain vehicle/route/location data and belong only in the
owner's private storage, not Git. This targeted log hardening is not an audit
of every plugin, script, HTTP endpoint or operating-system credential store.

## Verification

- `python3 tools/aaw1/run_focused_tests.py`: **20 native Rust tests PASS**;
  actual production logger/privacy/file/metadata/protobuf modules, with only
  MITM transport context stubbed. Primary dependency versions pinned to the
  product lockfile. Includes injected failing child output with a synthetic
  secret, fragmentation/overflow/mismatch/TTL/live-setting cases, memory/count
  limits, PNG summary, private-file content/permissions and opaque identity.
- `python tools/aaw1/test_hardening_wiring.py`: **5 source-integration guards PASS**;
  web save/error-path binding, logger-before-hook order, diagnostics-only live
  settings and opt-in capture config. These are not runtime HTTP/vehicle tests.
- `python tools/aaw1/test_sdr_wire.py`: **7 schema/wire tests PASS**, retaining
  #11's 21-field changes, UTF-8 compatibility and malformed/unknown-field checks.
- Rust formatting and staged diff checks. ARMv7 product build requested next by
  the owner; exact final SHA/run/ELF/NAS/notification belong in khud/Hub results.

Remaining final gates: full application/device execution; exact installed ELF;
real-HU SDR acceptance, dynamic service map, actual legacy/modern navigation,
PNG contents/lanes, GPS samples, reconnect/long-run behavior and #7 flicker.
Fragmented SDR rewriting is not introduced here; the known failing 542B sample
is single-frame. Bounded capture can retain fragmented original messages for
analysis. No protocol-version forcing, start injection, guessed channel, CAN
transmission, KHUD APK/ESP32 build or extra hardware dependency is introduced.

Five-view review: user request/real-car evidence; fixed parent/main source;
#7→#11→this stack; focused tests vs product build vs vehicle evidence; original
wire/presentation/PNG/OEM-display boundaries. No raw car packets/credentials in
fixtures. Old attached docs patches are superseded and are not applied.
