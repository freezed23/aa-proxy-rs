## 2026-10-31 #5 USB FunctionFS 주의사항 — 과거 v1.1 상태를 현재 상태로 사용 금지

- 이전 [AA docs PR #5](https://github.com/freezed23/aa-proxy-rs/pull/5)에서 지적한 **v1.2 FunctionFS 기반 USB 가젯과 v1.1 rootfs의 구형 `S92usb_gadget`/`umtprd` 시작 경로 충돌 가능성**은 정적 설계상 유효한 **바이너리만 교체할 때의 주의사항**이다. 기존 v1.1 rootfs의 가젯 점유 여부를 확인하지 않고 v1.2 실행 파일만 덮어쓰지 않는다.
- **현재 사용자 상태가 다르다:** 소유자는 장치 v1.2로 복구 후 Windows DHU / AA TBT 수신 및 protobuf 로그 추출을 다시 확인했다. 따라서 #5의 '현재 기기 실행 파일 = v1.1/fc461e8', 'v1.2 기기 실행 증거 없음', '1.2 이미지 설치 예정'이라는 문장은 **10/30 당시 체크포인트로만 해석**한다. 현 장치의 실제 `/usr/bin/aa-proxy-rs` SHA는 별도로 비교하지 않았으며, fork draft #7의 IDR 코드를 실제 설치된 것으로 주장할 수 없다.
- KHUD/Hub [2026-10-31 PR 정리 기록](https://github.com/freezed23/ai-project-hub/blob/main/projects/hud-navigation/PR_RECONCILIATION_20261031.md)에 따라 원본 #5는 직접 병합 대신 수정된 사실만 이 문서에서 보존하고 **INTEGRATED/SUPERSEDED** 분류한다. 본 문서 PR은 **docs only**, 바이너리/이미지/USB 시작 스크립트 수정·빌드·플래시·재부팅 없음.


> **OWNER CORRECTION (2026-10-31):** **과거에는 TMAP 내장 모의주행** 중 클러스터 TBT가 기대대로 표시되지 않았고, **현재는 Android Mock Location으로 GPS를 이동시키는 별도 시험을 진행 중**. 일반 경로 안내 시작 때 관측된 `0x8006/0x8007`과 Mock Location 주행 도중 연속 갱신 여부(아직 로그 미제공)를 구분한다.
## 2026-10-31 Windows DHU NAVIGATIONSTATUS 벤치 성공 — 기존 빌드/런타임 상태 구분

- **소유자 기존 AAW1 CS317 + Windows DHU 2.0-windows + Galaxy S23**: Zadig WinUSB로 USB `18D1:2D00` 드라이버 오류를 정리한 후 Android Auto 메인 화면 및 Instrument Cluster 표시 확인. DHU `instrumentcluster=true`, `navcluster=true`에서 `ServiceDiscoveryResponse`에 `navigation_status_service` 등장, 이번 세션 서비스 채널 `0x06`. `0x8003` ID, `0x8006` TURN_NORMAL_RIGHT=8/"교차로", `0x8007` 128m/32s/47km/2,836s/ETA20:51 복호화 Protobuf. **OWNER_DEVICE_OBSERVED: WINDOWS_DHU_BENCH**, 다른 HU/실차/장시간/차선/안내 종료/시뮬레이션 TBT는 아직 검증되지 않음.
- 재현·원본 일부 패킷·개인정보 주의: [AAW1_DHU_NAVSTATUS_BENCH_20261031.md](AAW1_DHU_NAVSTATUS_BENCH_20261031.md). 명시적으로 AA **소스 PR #7** `e5b180f`는 여전히 OPEN DRAFT/UNMERGED이고 owner-build `41186815239` 성공 ELF/NAS 해시는 **장치 현재 바이너리 해시와 미대조**. 이 벤치가 USB FunctionFS HUR stall 전체 해결이나 keyframe toggle의 실제 효력까지 증명하지 않는다. KHUD 제품에 추가 proxy HW 의존성 도입 아님.
- 아래 기존 10/30 인수인계의 `원격 자동 빌드 미구현`/ `현행 v1.2 미빌드` 같은 당시 표현은 **역사적 체크포인트**이며 이 10/31 후속 섹션의 self-hosted AA 빌드 및 사용자 벤치 관측으로 보완된다. 새로운 AA/KHUD 빌드나 배포/설치 동작 없음.

## 2026-10-30 원격 AA 실행 — 구현 병합 및 환경 preflight PASS

기존 수동 전용/TODO 정책은 소유자의 새 원격화 요청으로 대체됐다.
AA 바이너리는 이 포크에서 빌드하며, 실행은 private `freezed23/khud`의 별도 **AA Owner Build**를 사용한다.
KHUD Owner Build/APK/ESP32 작업과는 구분한다. 포크에 새 러너를 등록할 필요는 없다.
구현 [khud #584](https://github.com/freezed23/khud/pull/584), AI 진입점 [포크 #3](https://github.com/freezed23/aa-proxy-rs/pull/3) 병합 완료.

GPT·Claude는 사용자의 명시 AA 빌드 요청 때 소유자 선택 ref를 전체40자리SHA로 해석하고,
연결된 `freezed23` 계정으로 private khud에 제목 `[AA BUILD]`, 본문 전체 `build <AA-full-SHA>` Issue를 만든다.
소스 선택이 없으면 현행 refs와 열린 PR을 확인하고 선택을 확인한다. 구형 v1.1.0/임의 main으로 대체하지 않는다.
`preflight`는 컴파일 없이 Docker와 NAS 환경만 검사한다.
[run41077085999](https://github.com/freezed23/khud/actions/runs/41077085999) SUCCESS:
Docker 연결 및 AA 전용 NAS 파일 쓰기·읽기 확인. 전체 바이너리 빌드·복사 해시 대조와 장치 runtime은 아직 미검증.

빌드 산출물은 실행/재시도별 전용 폴더에 보관한다.
실행 파일·SHA256SUMS·BUILD_STATUS.json·build.log를 기록하고 ELF32 ARM 및 NAS 파일별 SHA256을 검사한다.
내부 호스트/PC 경로와 러너 운영 세부는
[private 운영 지침](https://github.com/freezed23/khud/blob/main/tools/aa-remote/README.md)에 둔다.

Telegram 완료 알림 코드 포함. 최초 점검 결과 **NOT CONFIGURED**:
private 봇 토큰과 numeric chat ID 등록 후 전송 가능하며 실제 SENT 증거는 아직 없다.
토큰을 공개 Git/Issue에 넣지 않는다. NAS·빌드·알림 결과를 구분한다.
빌드 요청만으로 장치 설치를 실행하지 않는다. 아래 기기 전송/백업/복구/5GHz 절차는 유지한다.

# AAWireless 1(CS317) — AAProxy 빌드·전송 인수인계

기록일: 2026-10-30 (KST)  
상태: **문서화 완료 / 현행 포크 v1.2.0 조합 미빌드 / 원격 자동빌드 미구현**  
범위: `freezed23/aa-proxy-rs`의 기존 AAWireless 1 CS317(RV1109, ARMv7) 벤치 전용. KHUD Android/ESP32 제품 빌드와는 별도 작업이다.

## 1. 요청 이름과 저장소 선택 — 최우선

| 사용자의 요청 | 대상으로 해석할 저장소 | 동작 |
|---|---|---|
| **"AA 빌드해줘", "aa-proxy 빌드", "AAW1 빌드"** | `freezed23/aa-proxy-rs` | CS317용 ARMv7 `aa-proxy-rs` 바이너리 빌드. **`khud` Owner Build 실행 금지.** |
| **"KHUD 빌드해줘", "4.45/2.95 빌드"** | `freezed23/khud` | 기존 KHUD 규칙/전용 self-hosted Windows Owner Build. AA 포크는 변경하지 않음. |
| 분석·문서 수정·PR 병합 | 관련 저장소만 | **명시적인 빌드 요청 전까지 컴파일·CI·기기 적용 금지.** |

- AA용 소스는 원칙적으로 소유자가 지정한 `aaw1/*` 개발 브랜치 또는 정확한 commit SHA를 우선한다. 지정된 대상이 없을 때는 **최신 `main`이나 구형 `aaw1/v1.1.0-5ghz`를 임의로 골라 빌드하지 않는다.** 최신 refs와 검증 범위를 확인한 후, 명확한 AAW1 후보가 없다면 빌드 소스 선택을 확인한다.
- 2026-10-30 스냅샷: 포크 `main@1a53ac1ec22b7dddf4f1933b9b29349e2341051a`, `Cargo.toml` version `1.2.0`. 기존 `aaw1/v1.1.0-5ghz@790c832d`는 **옛 실기 검증 브랜치**이며 현재 1.2.0 바이너리 산출물이라는 뜻이 아니다. 추후 새로운 `aaw1/*` 브랜치는 소유자 선택과 별도 구현/검증에 따른다.
- `main`에는 [포크 PR #1](https://github.com/freezed23/aa-proxy-rs/pull/1)을 통해 `Dockerfile.aaw1`, `tools/aaw1/build.sh`, 업스트림 hostapd 이슈 **초안**이 병합됐다. 구형 `src/config.rs`로 되돌리지 않았고, 5GHz 정수 주파수 감지 코드는 이미 `main`에 있다.

## 2. 실행 환경/산출물 (현재 **수동**)

- 현행 실행 방식: 소유자 PC의 **WSL2 Linux + Docker (Linux 컨테이너)**. GitHub Actions 워크플로는 이 포크에서 확인되지 않았다. `khud-pc-01`가 포크 저장소의 러너로 등록되어 있는지는 **미확인**이다. 따라서 지금 "원격으로 AA 빌드" 가능한 시스템이 설정되었다고 표현하지 않는다.
- 실제 스크립트: [`tools/aaw1/build.sh`](./build.sh). 내부에서 `Dockerfile.aaw1`을 사용해 `armv7-unknown-linux-gnueabihf` 타깃, `--no-default-features --features wasm-scripting`로 빌드하며 `io-uring`을 제외한다. 이것은 **RV1109 32비트 CS317**용이지 ARM64나 KHUD ESP32 이미지가 아니다.
- 실행 예시 (**아래는 향후 명시적인 빌드 요청 때만 수행**):

```sh
# WSL2, 포크 저장소 작업 디렉터리에서 실행
cd /path/to/aa-proxy-rs
git fetch origin
git switch <owner-selected-aaw1-branch>
git pull --ff-only
git rev-parse HEAD
./tools/aaw1/build.sh

# 빌드 성공 후 검증
file output/aa-proxy-rs
sha256sum output/aa-proxy-rs
(cd output && head -n 1 SHA256SUMS | sha256sum -c -)
```

- 출력: **`<repo>/output/aa-proxy-rs`** (실행 파일), **`<repo>/output/SHA256SUMS`** (첫 줄 파일 해시, 둘째 줄 `source: <branch>@<12자리SHA>`). 스크립트는 저장소 루트에서 상대경로 `output/`에 저장하므로, 해당 PC의 절대 경로는 **현재 WSL checkout 위치를 확인해야 한다**. 임의의 Windows `K:\ci\out` 또는 KHUD `output`을 AA 빌드 결과 경로라고 쓰지 않는다.
- `SHA256SUMS` 둘째 줄은 표준 checksum 형식이 아니므로, 위처럼 **첫 줄만** `sha256sum -c`에 넣는다. 바이너리 시작 로그의 짧은 git 레이블만으로 정확한 소스 SHA를 확정하지 말고 `git rev-parse HEAD`와 해시를 별도 기록한다.
- **미검증:** 1.2.0 `main`으로 이관된 새 조합은 아직 컴파일·ELF 로더/동적 라이브러리 호환성·실기 실행을 재검증하지 않았다. 기존 성공 보고는 `v1.1.0+5GHz 패치` 소유자 WSL2 빌드에만 해당한다.

## 3. 기기 전송 및 적용 절차 — 승인 후 수동 작업

대상은 소유자의 **AAWireless 1 CS317**, 접속 예시는 기존 벤치 내부 주소 `root@10.0.0.1`이다. 실제 접속 IP·기기 모델·현재 바이너리 버전·파일 해시를 **매번 확인**한다. 아래 명령은 **설명용이며 이번 문서 작업에서는 실행하지 않았다.** 전체 eMMC 순정 백업·기존 실행 파일·로컬 설정을 보존하고, 장치를 정상 복원할 수 없는 상황에서는 교체하지 않는다.

1. **사전 검사:** 새 `output/aa-proxy-rs`가 ARMv7 ELF인지 `file`/`readelf -h`로 확인하고 로컬 SHA256을 기록. 원본 `/usr/bin/aa-proxy-rs`는 덮어쓰기 전에 `/data/`에 백업. 기기 설정(`/etc/aa-proxy-rs/config.toml`, `/etc/hostapd.conf.in`, `/etc/init.d/`)은 **장치 내에만** 별도 보존하고 비밀정보를 GitHub에 올리지 않는다.
2. **전송:** Windows/OpenSSH의 기본 `scp`는 SFTP 서버를 요구할 수 있다. 장치에 `/usr/libexec/sftp-server`가 없는 경우 **`scp -O`**(레거시 SCP)를 먼저 사용. 원격 `scp` 프로그램도 없으면 WSL2의 표준입력 스트림으로 전송한다.

```sh
# WSL2/Linux에서 (빌드 결과의 체크아웃 경로 기준)
scp -O output/aa-proxy-rs root@10.0.0.1:/data/aa-proxy-rs.candidate

# scp -O도 실패한 경우의 대안 (위 명령과 둘 중 하나만)
cat output/aa-proxy-rs | ssh root@10.0.0.1 'cat > /data/aa-proxy-rs.candidate'
```

3. **검증:** `ssh root@10.0.0.1`로 접속해 `sha256sum /data/aa-proxy-rs.candidate`를 로컬 해시와 **완전히** 비교. `file`, `ldd`/ELF 로더 검사 가능 여부는 기기 명령 구성을 확인. 결과가 다르거나 동적 로더 불일치면 **중지**.
4. **수동 교체:** 장치 내 init 스크립트 `/etc/init.d/S93aa-proxy-rs`가 실제 존재하고 `stop/start` 지원을 확인한 뒤에만 서비스 정지 → 백업 → `/usr/bin/aa-proxy-rs` 교체 → 실행권한 → `sync` → 재시작. 스크립트가 다르면 실제 init 체계에 맞게 수정하고, 실행 중 바이너리 덮어쓰기를 강행하지 않는다.

```sh
# 장치의 SSH 셸 (S93 스크립트의 존재/stop/start 지원을 먼저 확인한 경우만)
cp -p /usr/bin/aa-proxy-rs /data/aa-proxy-rs.before-aa-build
/etc/init.d/S93aa-proxy-rs stop
cp /data/aa-proxy-rs.candidate /usr/bin/aa-proxy-rs
chmod 755 /usr/bin/aa-proxy-rs
sync
/etc/init.d/S93aa-proxy-rs start
tail -50 /var/log/aa-proxy-rs.log

# 실패 시 롤백: stop → cp /data/aa-proxy-rs.before-aa-build /usr/bin/aa-proxy-rs
# → chmod 755 → sync → start; 원본 백업은 성공 확인 전까지 삭제하지 않음.
```

5. **기기 확인:** 첫 부팅/서비스 로그의 버전과 SHA, BT/Wi-Fi/TCP/TLS, 5GHz 채널 36의 `AP-ENABLED`, HUR 영상·연속 전송·반복 연결을 별도 검증. 실제 보조 화면/실차 검증은 **별도의 증거 단계**다. 바이너리 빌드만으로 완료 판정하지 않는다.

### 5GHz 주의사항

- 신규 바이너리의 정수 주파수 파서만으로 기존 `config.toml`의 `band = "2.4"` / `channel = 6`을 자동으로 고치는 것은 아니다. 또한 `aa-proxy/buildroot`의 AAW 공통 `hostapd.conf.in`에 남아 있는 `TKIP`·`[HT40+]`가 CS317 AP를 막을 수 있다.
- 소유자 벤치에서는 **장치 내** 설정을 `band="5"`, `channel=36`, CCMP, `wmm_enabled=1`로 수정하고 HT40+를 제외해 기본 hostapd 경로가 올라왔으며, 임시 `S91hostapd5g` 우회는 비활성화했다. **새 이미지/업데이트 후에는 파일 상태와 복귀 여부를 재검증**해야 한다.
- 관련 근거: KHUD [`docs/KHUD_AAW1_AAPROXY_RS_BENCH_20261030.md`](https://github.com/freezed23/khud/blob/main/docs/KHUD_AAW1_AAPROXY_RS_BENCH_20261030.md) 10절 및 [`tools/aaw1/UPSTREAM_ISSUE_hostapd_template.md`](./UPSTREAM_ISSUE_hostapd_template.md). 업스트림 이슈는 **초안, 미제출**.

## 4. 향후 원격 빌드 가능 조건 (현재 **TODO**, 구현 금지)

- 포크에 별도 `.github/workflows`와 수동 트리거가 있어야 한다. **이번 요청에서는 생성하지 않는다.**
- `khud-pc-01`에 **포크 저장소에서도 동작하는 self-hosted Windows 러너**가 등록되어 있는지 확인해야 한다. 기존 KHUD 러너가 다른 저장소에서 자동으로 사용 가능하다고 가정하지 않는다. 실제 등록은 소유자가 로컬 PC에서 처리해야 한다.
- 이후 소유자가 별도로 승인하면 포크 전용 작업에서 WSL2/Docker 빌드를 실행하고 로컬 `output/`에만 결과를 두는 방식을 검토한다. **GitHub-hosted runner·upload-artifact·KHUD Owner Build를 기본 경로로 사용하지 않는다.** 러너/보안·워크플로/경로/재현성은 아직 미구현이다.

## 5. 다음 AI 체크리스트

- [ ] 요청이 AA인지 KHUD인지 먼저 분리하고, **명시적인 빌드 지시**와 선택 ref/commit 확인.
- [ ] 포크 현재 `main`과 열린 PR/`aaw1/*`를 확인. 오래된 `aaw1/v1.1.0-5ghz`를 최신 1.2.0으로 착각하지 않는다.
- [ ] 빌드 전: WSL checkout SHA, `Dockerfile.aaw1`/`build.sh` 입력, Docker 가용성과 러너 등록 여부 확인. 새 환경에서 빌드 성공을 추정하지 않는다.
- [ ] 빌드 후: 파일 위치·바이트 크기·SHA256·ELF architecture·의존 라이브러리·실행 로그·장치 재시작 결과를 단계별 기록.
- [ ] **빌드 지시만**으로 기기 플래시/교체, CAN 송신 또는 KHUD APK/ESP32 빌드를 실행하지 않는다. 기기 설치/교체는 소유자가 별도로 승인한 후에 진행.
- [ ] secret/키/인증서/SSID 비밀번호/순정 eMMC 원시 덤프를 코드·문서·CI 로그에 넣지 않는다.

원본 이력: 포크 [#1](https://github.com/freezed23/aa-proxy-rs/pull/1) (1.2.0 도구 병합), KHUD [#581](https://github.com/freezed23/khud/pull/581) 및 [#582](https://github.com/freezed23/khud/pull/582), Hub [#280](https://github.com/freezed23/ai-project-hub/pull/280) 및 [#281](https://github.com/freezed23/ai-project-hub/pull/281).
