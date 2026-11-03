# ENUM web toggle and persistence

Base: exact PR19 source 7e42de7, separate branch/PR; not aa-proxy main. No PR14-17 connection changes reintroduced.

The embedded web schema lacked nav_sdr_enum_to_phone_enabled, so web form bulk-save omitted it. Bulk POST deserialized missing fields to default false, replacing live configuration. This is a source-proven reset path, not proof of the owner's particular reboot event. Prior /etc and /data/etc observations describe OverlayFS views, not two independently selected configs.

Add advanced ENUM toggle (default OFF, independent of version override), merge only supplied top-level JSON fields onto locked live configuration (explicit false honored), persist before publishing memory and return HTTP failure on storage error. Single-entry updates validate before storage. All current config writers use same-directory atomic replacement plus file and parent-directory sync; mode0600. BT token provisioning publishes memory/auth success only after persistence. Web save re-GETs configuration, restores controls from server and verifies ENUM read-back. Reconnect AA to renegotiate SDR.

Host gate uses actual production merge/write helper: omission and explicit false, invalid JSON root, replacement/reload/permissions, failed-rename preservation/temp cleanup. Existing focused protobuf/diagnostic gates retained. ARM compile checks integration; these focused tests are not full HTTP/BT/browser or AA1 OverlayFS/power-loss tests. No device install or automatic AA reconnect. No system mount/boot script changes. Settings edited externally while process is running are not a supported live-update path; use web or stop/edit/start.
