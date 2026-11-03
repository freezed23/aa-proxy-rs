# PR13 baseline with observational navigation parser (2026-11-03)

Owner reports AA connection failure on newer source and requests PR13-based rebuild.
Base: exact PR13 `705ed837dc31dd24f32d1fad1b2414aac9712dd5`. No PR14/15 AARE version gating, WPP/startup policy, short-write changes, PR16 Google-AA-only gates, or PR17 gate restoration imported. Causality of the reported failure remains UNKNOWN.

Only additions: bounded nav_capture and NAV_PARSED/NAV_PARSE_ERROR observer for actual SDR navigation channel at HU egress, before optional suppression. Preserve protobuf presence/lanes/directions/highlights/unknown fields, never create fields. Both context initializers registered. Short continuation fragments reach observer and existing suppression before generic guard. Limits64KiB/128frames/3s. No schema, transport, config defaults or dependencies changed. PR13 SDR/companion/display behavior retained.

Version adjustment ALREADY exists in PR13; no new version forcing. Defaults override=false/drop=false; stored settings persist across binary replacement. For official-AA baseline inspect effective local settings: empty aa_server_tcp_addr selects normal phone listener, override=false preserves HU negotiation. No device settings automatically cleared. DHU1.7 does not need forcing1.6. Legacy-HU1.6+modern-drop is a separate optional test after baseline connection.

Validation: five-path diff only; proxy.rs/bluetooth.rs/config/proto/transport/dependencies unchanged. Builder-host collector3 tests required before ARM export. Immutable-source owner build follows. No install/settings/OTA/CAN TX/new road action. Connection recovery and lane output remain UNVERIFIED.
