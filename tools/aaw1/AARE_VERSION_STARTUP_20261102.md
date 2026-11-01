# AARE initial version delivery repair (2026-11-02)

Status: IMPLEMENTED / NOT BUILT / NOT DEVICE VERIFIED.

Conversation search recovered the earlier failure text `session failed/closed: no version request from head unit`, FunctionFS transfer counters 0 B in both directions and subsequent read/stall timeouts. Official Android Auto worked through the same AA1/DHU. The owner also reports failure with the vehicle and certificate failure without AA1. These are retrieved historical reports, not a newly inspected raw session or proof of the exact USB root cause. VERSION_REQUEST itself is a normal HU-initiated message.

Source basis: aa/main b2921134; this repair is stacked on #13 at 705ed837 (which includes #12 capture hardening), not merged into main. AARE/main e8da04dc. Existing #6/#7/#11/#12/#13 branches remain intact. No protocol override/filter setting is enabled or device setting changed here.

Changes:
- Direct `aa_server_tcp_addr` mode reads exactly the real first unfragmented plaintext channel-0 VERSION_REQUEST before connecting to the phone server. Short reads are accumulated, EOF fails, a 15s whole-frame deadline applies, no trailing TLS frame is consumed. The original request is passed once through MITM or passthrough; no synthetic request or guessed version.
- Both MITM directions validate request/response ID, channel, flags, minimum fixed-prefix length and response success status. A 15s stage deadline and explicit VERSION receipt/transmission logs distinguish HU silence, relay closure and phone response failure. Shared session cleanup closes both sides after failure.
- Pretty printing reads the request's 4-byte version prefix and response's 6-byte version/status prefix before optional protobuf options.
- Companion AARE repair keeps the existing one-shot LAN listener after an error while closedDuring == VERSION; explicit Stop and post-version session termination retain the existing one-shot behavior. Certificate trust behavior is unchanged.

Limits:
- Direct TCP gating does not apply to phone-initiated wireless TCP mode. Validation/logging does; neither manufactures a missing HU request.
- This does not prove a fix for the 0 B FunctionFS/USB failure. It avoids starting AARE's version timer against a silent HU in direct mode, preserves LAN retry, and makes failures diagnosable.
- TLS retry/fallback policy is unchanged. Check effective MITM vs boot-wide fallback before interpreting a later certificate failure.
- Six regression tests authored for short reads/no read-ahead, EOF, packet validation, relay closure and pretty prefixes. Not executed: owner explicitly has not requested compilation/build. No Cargo/Gradle/Actions/OTA/install/road test. `git diff --check` passed. Tree-sitter grammar checks passed for all three changed Rust and both companion Kotlin files; grammar checks are not compilation/type checks. Rustfmt was unavailable locally.

Five review views: retrieved owner evidence vs raw-log limits; main vs stacked source; open branch ancestry; source-only vs build/runtime; version framing/transport vs TLS trust and KHUD/CAN boundaries.
