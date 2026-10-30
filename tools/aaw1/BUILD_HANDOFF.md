## 2026-10-30 원격 빌드 추가 — 아래 수동 전용/원격 TODO는 이 절로 대체

사용자가 원격화·NAS 자동 복사·Telegram 연결을 새로 요청했다.
[AA Owner Build](https://github.com/freezed23/khud/blob/main/.github/workflows/aa-owner-build.yml)는
private `freezed23/khud`의 기존 Windows 러너를 재사용하는 **AA 전용** 작업이다.
KHUD Owner Build/APK/ESP32 실행과 다르며 이 AA 포크의 별도 러너 등록은 필요 없다.
구현 병합: [khud #584](https://github.com/freezed23/khud/pull/584), `e2678ca6dbae20fa4ec84a87155e9b9738839ba2`.
운영 상세: [tools/aa-remote/README.md](https://github.com/freezed23/khud/blob/main/tools/aa-remote/README.md).

- GPT·Claude: 사용자의 명시적 **AA 빌드** 요청 때 이 저장소의 소유자 선택 ref를 전체40자리SHA로 해석한다.
  선택이 없으면 현행 main/aaw1/*와 미병합 PR을 확인하고 소스를 확인한다. 구형 v1.1.0/임의 main으로 대체하지 않는다.
- 연결된 GitHub 계정 `freezed23`로 **khud**에 Issue 생성: 제목 `[AA BUILD]`, 본문 전체 `build <AA-full-SHA>`.
  Actions의 **AA Owner Build** 결과·Issue 댓글을 확인. `[KHUD GS BUILD]`를 사용하지 않는다.
- 설정 점검은 같은 제목에 본문 `preflight`: Docker 엔진/NAS 읽기·쓰기만 검사, 컴파일 없음.
- AAW1 ARMv7 Docker export → ELF32 ARM 검사 → 정확한 SHA/크기/해시 기록 →
  PC `K:\\ci\\out\\aa-proxy-rs\\run<id>-attempt<n>` 및 NAS `\\\\freezed-4t\\homes\\GPT\\AAProxy\\run<id>-attempt<n>`.
  모든 복사 파일 SHA256 대조, NAS 실패는 작업 실패로 보고하고 PC 파일 보존.
- Telegram은 PC private JSON 또는 `AA_TELEGRAM_BOT_TOKEN`/`AA_TELEGRAM_CHAT_ID` secrets에 설정한 경우만 전송.
  토큰이 없으면 NOT CONFIGURED. 빌드/NAS/알림 결과는 별도로 보고한다.
- 구현과 실제 Docker 가용성·새 바이너리 BUILT·장치 실행은 구분. 최초 preflight는
  [khud issue #585](https://github.com/freezed23/khud/issues/585)에서 확인한다. 아직 성공을 추정하지 않는다.
- 아래의 기기 전송·백업/복구·5GHz 주의사항은 유지한다. 원격 빌드만으로 장치 설치를 승인한 것이 아니다.

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
