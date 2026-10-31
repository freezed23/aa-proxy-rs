# AAWireless 1 CS317 — Windows DHU USB + Android Auto NavigationStatusService TBT 벤치 (2026-10-31)

상태: **OWNER_DEVICE_OBSERVED / WINDOWS_DHU_BENCH**. KHUD 제품 통합, 실차, 장시간/재연결 시험, Windows DHU 외 헤드유닛 TBT 지원 증명 **아님**. 이 문서는 소유자의 기존 AAWireless 1 장치 연구용 기록이며 KHUD에 추가 프록시 하드웨어를 제품 의존성으로 도입하지 않는다.

## 1. 출처·소프트웨어 상태 구분

- 장치: 소유자 AAWireless 1 **CS317 / RV1109 ARMv7**. Galaxy S23이 Android Auto 전화 측, Windows PC의 Google Desktop Head Unit **2.0-windows**가 USB head-unit 측. AAW1에서 `aa-proxy-rs` v1.2 **FunctionFS** 경로 확인. 정확한 **장치 실행 바이너리 SHA256을 읽거나 대조하지는 않았다**.
- 독립 빌드 증거: 포크 **OPEN DRAFT/UNMERGED** [#7](https://github.com/freezed23/aa-proxy-rs/pull/7) source `e5b180f760ecb938a86a0c3b85f4eb50a60ac61a` = v1.2.0 + upstream @manio `9aca104c` opt-in keyframe toggle, [self-hosted AA Owner Build run 41186815239](https://github.com/freezed23/khud/actions/runs/41186815239) SUCCESS, ELF32 ARM SHA256 `79494f54d91373926df6f4eec79e3f6e6c71b438e3354fb2f7e8ae76e88c3d55` and NAS copied hash VERIFIED. **CI output hash != installed binary identity** without device `sha256sum /usr/bin/aa-proxy-rs`. The observed device protocol features are consistent with the v1.2 code path, not a pin to that exact commit.
- AA fork `main@f3628482bf14df478deb0d89f0885943f0ce2622` remains v1.2 base and does NOT include the unmerged keyframe PR #7. Prior v1.1 `fc461e8` remains independent rollback reference; do not silently merge v1.1 and v1.2 patches.

## 2. USB failure boundary and successful Windows DHU connection

1. Early owner HUR/FunctionFS trials reached phone BT AA handshake and Wi-Fi/TCP, then `accessory gadget bound (vid=18d1,pid=2d00)` at UDC `ffd00000.dwc3`, but lacked `accessory bulk pipes ready` and watchdog showed `ffs_epfile_ioctl`. Repeated HUR stall/reconnect is **not established fixed**.
2. On Windows, USB enumeration showed `aa-proxy (AAWireless 1)` at `USB\\VID_18D1&PID_2D00` with **Error / driver not installed**, after an initial Zadig WinUSB attempt. AAW1 UDC `addressed` (not `configured`) was observed during the failure.
3. Owner used Zadig to repair/select **WinUSB for the *18D1:2D00* AOA device**, not unrelated USB dongles; Windows DHU subsequently displayed the AA main screen. This sequence supports a Windows driver obstacle; it does **not** independently isolate every earlier FunctionFS watchdog root cause.
4. Running AA relay later showed phone→HU **22.9 MiB**, HU→phone **159.6 KiB** cumulative (~2m32s in one session). Data relay observed, not 24-hour/long-run stability.
5. Working owner config noted `legacy=false`, `dhu=false`, `mitm=true`; `dhu=false` is correct for **real USB DHU `--usb`**. `dhu=true` selects a distinct TCP emulator mode and should not be confused with Windows Desktop Head Unit. The current `change_usb_order` value should be read from the device on any future session; the earlier successful logs initialized USB after phone BT/TCP.

Example Windows DHU launch from the Android SDK `extras/google/auto` folder:

```powershell
@'
[general]
touch = true
resolution = 800x480
dpi = 160
framerate = 30
instrumentcluster = true
navcluster = true
'@ | Set-Content "$env:TEMP\\aa-navcluster.ini" -Encoding ascii
.\\desktop-head-unit.exe -c "$env:TEMP\\aa-navcluster.ini" --usb
```

**Critical difference:** with the earlier default DHU config, `ServiceDiscoveryResponse(0x0006)` advertised channels 1..7 plus proxy vendor channel 8 but **no** `navigation_status_service`. After enabling the cluster options, service discovery included `navigation_status_service`; **this session's ID was 0x06**. Service ID is dynamic and must be resolved from each current 0x0006 response, not hardcoded.

## 3. Owner-confirmed decrypted NavigationStatus payloads

The owner enabled filtered packet logging on the device (`/etc/aa-proxy-rs/config.toml`, local log `/data/aa-proxy-rs.log`). Effective values were `pkt_debug=true`, `hexdump_level="DecryptedOutput"`, `pkt_debug_filter_enabled=true`, `pkt_debug_filter_service_kinds="control,navigation_status"`, `pkt_debug_filter_message_ids="0x0006,0x8001,0x8002,0x8003,0x8004,0x8005,0x8006,0x8007"`, pretty-proto true and full-frame true. **Owner additionally captured `mitm/MD DecryptedInput` packet dumps**; observed direction should come from the packet log, not inferred from hexdump config alone.

| Message | Owner-provided evidence | Certainty |
|---|---|---|
| `0x0006` `MESSAGE_SERVICE_DISCOVERY_RESPONSE` | After cluster enabled, `navigation_status_service` in proto; this session ch `0x06` | BENCH_OBSERVED |
| `0x8003` `INSTRUMENT_CLUSTER_NAVIGATION_STATUS` | Received at ~00:02:34.789/798 device-relative time | ID only; status payload **not provided** |
| `0x8006` `INSTRUMENT_CLUSTER_NAVIGATION_STATE` | `steps[0].maneuver.type=TURN_NORMAL_RIGHT` numeric **8**, `road.name="교차로"`, `cue.alternate_text="교차로"` | DecryptedInput + parsed proto + owner DHU cluster screenshot |
| `0x8007` `INSTRUMENT_CLUSTER_NAVIGATION_CURRENT_POSITION` | step distance **128 m**, time to step **32 s**, destination **47,000 m** / displayed **47 km**, ETA **20:51**, remaining **2,836 s** | DecryptedInput + parsed proto + owner DHU cluster screenshot |

Selected raw wire payloads (hex bytes from owner-supplied **decrypted** logs; no account/private road text beyond generic cue):

```text
0x8006 80 06 0A 1E 0A 02 08 08 12 0B 0A 09 EA B5 90 EC B0 A8 EB A1 9C 22 0B 0A 09 EA B5 90 EC B0 A8 EB A1 9C
0x8007 80 07 0A 0E 0A 0A 08 80 01 12 03 31 32 38 18 01 10 20 12 16 0A 0A 08 98 EF 02 12 02 34 37 18 02 12 05 32 30 3A 35 31 18 94 16
```

No `NavigationLane` entries in this sample, no repeated maneuver-enum family coverage and no route-end/status transitions yet. The owner clarified that the ongoing simulated drive uses **Android Mock Location (synthetic GPS movement)**, not necessarily the navigation app's built-in simulation function. The `0x8006/0x8007` sample was captured when ordinary route guidance was started. **Whether new `0x8006/0x8007` messages continue to update as Mock Location coordinates move is NOT YET VERIFIED**; a lack of cluster changes at a prior observation does not prove Mock Location suppresses the AA navigation service. Capture timestamped consecutive messages before/during/after the mock movement and route-end `0x8003` to resolve.

Packet-oriented spot check on device (never post unredacted full config or Wi-Fi credentials):

```sh
grep -Ei 'navigation_status_service|service_kind=navigation_status|message_id = 800[367]' /data/aa-proxy-rs.log | tail -n 50
```

`1970-01-01` timestamps on boot logs reflect the device RTC before clock sync; their elapsed deltas are still diagnostic.

## 4. Product boundary and next diagnostic gate

- The **AA service metadata**, numeric TBT direction/distance/ETA, and MITM packet parse are bench-confirmed. KHUD product transport/source arbitration is not implemented or approved by this study. The other party's host must advertise `NavigationStatusService`, so Windows DHU success does not establish support in a given OEM HU or Android Auto dongle.
- Static AA `NavigationManeuver.NavigationType` **0..42 (43/43)** vs KHUD TMAP/KNSDK/M-CAN: [KHUD source audit](https://github.com/freezed23/khud/blob/main/docs/AA_TBT_PONTUS_IG_CROSSWALK_20261031.md), [CSV](https://github.com/freezed23/khud/blob/main/docs/data/aa_navigationtype_khud_crosswalk_20261031.csv). Numeric spaces differ; `roundabout_exit_number` is not a KHUD clock-angle visual; lane `is_highlighted` is not proven `nextPreferred`.
- Next bench captures should preserve `0x8003` raw status values, route end/reroute, multiple maneuver types, roundabout `exit_number/exit_angle` and lanes. Avoid exposing SSID/password/phone MAC/GPS/real destination. Do **not** install to vehicle, add proxy board as KHUD product requirement, modify CAN TX, or run any build without a separate explicit owner request.

**Five views:** owner DHU screenshot+decrypted log; fork 1.2 static proto/source; Windows USB VID/PID and WinUSB sequence; CI built vs device binary identity; KHUD source/provider/policy and real-road proof are separate.
