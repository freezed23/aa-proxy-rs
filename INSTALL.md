# INSTALL.md — AAW1 (CS317) aa-proxy-rs 설치/검증/복구

> 대상: 소유자 AAWireless 1 CS317, ARMv7 ELF32. 이 파일은 **사용자 수동 설치** 절차입니다. 빌드만으로 기기에 설치되지 않습니다. 실제 HU에서 1.6 전환/차단의 안전성이 검증되지 않았으므로 주차된 벤치에서만 설정 변경 및 시험하세요.
>
> 이번 기능: `modern_nav_drop_before_hu`는 **기본 false**. AA 현대식 `0x8006/0x8007`을 NavigationStatus 채널에서 선택적으로 HU 전달 차단. 다른 채널, `0x8001/02/03`, H.264, 오디오, 터치 및 AA 포커스 메시지는 통과합니다. **PDK/AA 버전 변경과 데이터 차단은 별개 설정**입니다. 위 버전을 강제해 생기는 기타 신규 메시지까지 모두 호환되지는 않습니다.

## 1. 빌드 결과 확인

- 빌드 소스는 정확한 40자리 Git SHA, 빌드는 private `freezed23/khud`의 **AA Owner Build** 워크플로와 `khud-pc-01` self-hosted Windows 러너를 사용.
- Owner Windows PC 기본 결과 폴더: `K:\ci\out\aa-proxy-rs\run<id>-attempt<attempt>\`
- Owner NAS: `\\freezed-4t\homes\GPT\AAProxy\run<id>-attempt<attempt>\`
- 결과: `aa-proxy-rs`, `SHA256SUMS`, `BUILD_STATUS.json`, `build.log`.
- `BUILD_STATUS.json`의 정확한 소스 SHA, 빌드 성공, ARMv7 ELF32와 파일 SHA256 대조. **이 문서의 예시 이름을 실제 run id로 오해하지 마세요.**

## 2. 기존 AA1 백업/무중단 사전 준비

PC PowerShell에서 (AA1의 실제 접속 IP를 먼저 확인. 예시 `10.0.0.1`):

```powershell
ssh root@10.0.0.1 'uname -m; sha256sum /usr/bin/aa-proxy-rs; ls -l /etc/init.d/S93aa-proxy-rs; ls -l /etc/aa-proxy-rs/config.toml'
```

- SSH 호스트키가 달라졌다면 무조건 신뢰하지 말고 기기 변경/초기화 여부와 새 키의 지문을 **독립적으로 확인**한 뒤 기존 known_hosts 항목을 정리.
- 기기 내부 비밀번호·키·SSID·MAC·설정 원문은 공개 GitHub Issue, PR, 로그에 올리지 말 것.
- 현재 기기 서비스 구성을 먼저 확인. `S93aa-proxy-rs`가 없거나 stop/start를 지원하지 않으면 아래 서비스 명령을 그대로 실행하지 말 것.

백업 (현재 실행 파일 및 설정을 장치에 보존):

```powershell
ssh root@10.0.0.1 'mkdir -p /data/aa-proxy-backup; cp -p /usr/bin/aa-proxy-rs /data/aa-proxy-backup/aa-proxy-rs.previous; cp -p /etc/aa-proxy-rs/config.toml /data/aa-proxy-backup/config.toml.previous; sha256sum /data/aa-proxy-backup/aa-proxy-rs.previous'
```

## 3. 새 바이너리 전송

PowerShell에서 실제 빌드 폴더로 이동 (`<RUN_ID>`와 attempt는 실제 결과로 교체):

```powershell
cd 'K:\ci\out\aa-proxy-rs\run<RUN_ID>-attempt1'
Get-FileHash .\aa-proxy-rs -Algorithm SHA256
scp -O .\aa-proxy-rs root@10.0.0.1:/data/aa-proxy-rs.candidate
ssh root@10.0.0.1 'sha256sum /data/aa-proxy-rs.candidate; ls -l /data/aa-proxy-rs.candidate'
```

- `scp`가 SFTP 서버 없음 오류를 내면 `scp -O`(레거시 SCP) 유지; 둘 다 불가능하면 WSL/Linux에서 `cat aa-proxy-rs | ssh root@10.0.0.1 'cat > /data/aa-proxy-rs.candidate'`.
- **PC와 기기 SHA256이 완전히 일치해야** 다음 단계로 이동.

## 4. 서비스 정지 후 교체

다음 명령은 **실제 `/etc/init.d/S93aa-proxy-rs`가 존재하며 stop/start가 지원될 때만** 실행:

```powershell
ssh root@10.0.0.1 '/etc/init.d/S93aa-proxy-rs stop; cp /data/aa-proxy-rs.candidate /usr/bin/aa-proxy-rs; chmod 755 /usr/bin/aa-proxy-rs; sync; /etc/init.d/S93aa-proxy-rs start'
ssh root@10.0.0.1 'sha256sum /usr/bin/aa-proxy-rs; pidof aa-proxy-rs; tail -50 /var/log/aa-proxy-rs.log'
```

- 교체 중 차량을 주행하지 말 것. 실행 중 바이너리 강제 덮어쓰기 금지.
- 현재 실행 파일과 `/proc/<PID>/exe`의 SHA256까지 일치하면 실제 프로세스 교체 확인 가능.

## 5. 설정: 기본은 호환 모드

기본(변경 전 정상 연결을 우선 확인):

```toml
protocol_version_override_enabled = false
modern_nav_drop_before_hu = false
```

실험 전용, 순정 IG 기기가 새 형식을 거부할 수 있음 (장치 로컬 config만 수정):

```toml
protocol_version_override_enabled = true
protocol_version_override_major = 1
protocol_version_override_minor = 6
modern_nav_drop_before_hu = true
```

- 변경 전 `/data/aa-proxy-backup/config.toml.previous` 보존. 설정은 **새 AA 세션 시작 전** 적용해야 하며, 설정 변경 후 AA 연결 재수립.
- **중요:** `0x8006/07` 차단만 구현한 것이므로, AA 1.6에서 나오는 다른 메시지 때문에 HU가 연결을 종료할 수 있음. 최초 검증은 주차된 DHU 또는 보호된 벤치 우선.
- 동적 NavigationStatus 채널은 SDR로 찾아야 함. `0x09` 고정 금지. SDR 파싱 실패 시 이 필터는 정상 동작을 보장하지 않음.
- `0x8006/07`은 기존 pretty-print/선택적 비공개 capture로 복호화 입력에서 확인 가능. **KHUD 실시간 WebSocket 송신은 이번 버전의 구현 범위가 아님.**

## 6. 시험 체크

1. 기본 비활성화 설정에서 AA 연결, Wi-Fi/BT/TLS, 메인 H.264 화면, 음악·통화·터치 확인.
2. 주차된 테스트 환경에서만 1.6 + 차단 활성화. 로그에서 SDR의 NavigationStatus service id와 현대식 메시지 **수신**, `modern navigation consumed before legacy HU` 차단 로그 확인.
3. `0x00FF`/연결 재설정/메인 영상 이상이 발생하면 **즉시 기존 설정으로 복귀**.
4. 구조화 TBT/차선 미수신은 실험 실패가 아니라 로그상 미확정 상태로 남겨야 함. OEM 계기판 표시, M-CAN 송신, 실차 장시간 안정성은 별도 검증.

## 7. 실패 시 되돌리기

```powershell
ssh root@10.0.0.1 '/etc/init.d/S93aa-proxy-rs stop; cp -p /data/aa-proxy-backup/aa-proxy-rs.previous /usr/bin/aa-proxy-rs; chmod 755 /usr/bin/aa-proxy-rs; cp -p /data/aa-proxy-backup/config.toml.previous /etc/aa-proxy-rs/config.toml; sync; /etc/init.d/S93aa-proxy-rs start'
ssh root@10.0.0.1 'sha256sum /usr/bin/aa-proxy-rs; pidof aa-proxy-rs'
```

원본 백업, 기존 동글의 BT/Wi-Fi/5GHz·hostapd 설정은 성공 확인 전까지 보존. 이 기능은 **휴대폰→HU NavigationStatus 차단**만 담당하고 순정 차량 CAN을 직접 송신하지 않습니다.

추가 근거: `tools/aaw1/BUILD_HANDOFF.md`, private `freezed23/khud/tools/aa-remote/README.md`, `docs/IG_AA_LEGACY_NAV_REAL_HU_LOG_20261101.md`.
