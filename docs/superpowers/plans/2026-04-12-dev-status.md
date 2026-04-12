# EDIE Minigames — 개발 현황 (2026-04-12)

## 세션 요약

이번 세션에서 3개 게임 전반에 걸쳐 대규모 작업이 진행되었습니다.
PR #15~#31 총 17개 PR이 merge되었습니다.

---

## 게임별 현황

### EDIE Runner — PLAYABLE

| 항목 | 상태 | PR |
|------|------|-----|
| Crossfire 패턴 수정 (lanes [260,320,340] + 교대 발사) | Done | #15 |
| Crossfire retain 범위 확대 (2발 생존) | Done | #23, #24 |
| 모바일 Pause 버튼 (Playing/BossFight) | Done | #16 |
| NameEntry 터치 컨트롤 5버튼 | Done | #16 |
| 리더보드 버전 업데이트 시 점수 유실 수정 | Done | #20 |
| edie-runner 브랜치 동기화 | **미완료** | #37 |

> **주의**: 위 수정들은 develop/main에만 반영됨. 배포는 edie-runner 브랜치 기준이므로 동기화 필요 (#37)

### EDIE Battle Reverse — BETA TEST

| 항목 | 상태 | PR |
|------|------|-----|
| M5 Aurora 파워업 3종 (DoubleStrike, VirusCure, ForceFlip) | Done | #19 |
| 4 AI 모드 (Easy/Normal/Hard/Insane) | Done | #19 |
| 턴 타이머 (난이도별 8~30초) | Done | #19 |
| 승리/패배 화면 (캐릭터 + 스코어) | Done | #19 |
| 모바일 뷰포트 1280x720 + Alice 비율 수정 | Done | #19 |
| Camera.with_logical() 추가 | Done | #19 |
| 아반떼N 타이머 아이콘 (car0.gif 17프레임) | Done | #25 |
| 보드 배경 (그라데이션 + 격자 + 코너 장식) | Done | #25 |
| HP바 캐릭터 아이콘 | Done | #25, #27 |
| 이스터에그 (로고 클릭 → 캐릭터 순환) | Done | #26, #27 |
| 모바일 UX (텍스트 그림자, 패널 배경, 대비) | Done | #27 |
| 이스터에그 로고 클릭 동작 확인 | **미확인** | #36 |

### EDIE 초능력 윷놀이 — BETA TEST

| 항목 | 상태 | PR |
|------|------|-----|
| M1 보드 29칸 그래프 + 윷 던지기 + 이동/잡기/업기 | Done | #22 |
| M2 렌더링 + 터치/키보드 입력 + 메뉴 + 게임오버 | Done | #22 |
| M3 16종 초능력 카드 시스템 | Done | #28 |
| WASM 빌드 배포 (deploy.yml) | Done | #29, #31 |
| M4 보드/말 에디 에셋 교체 | **미완료** | #32 |
| M5 SFX + 윷 던지기 애니메이션 | **미완료** | #33 |
| M6 AI 상대 | **미완료** | #34 |
| M7 모바일 UX 최적화 | **미완료** | #35 |
| M8 온라인 멀티플레이 | **미완료** | #39 |

### 인프라 / 공통

| 항목 | 상태 | PR |
|------|------|-----|
| README 업데이트 (게임 링크, 팬게임 공지) | Done | #15, #18 |
| 랜딩 페이지 (에디 에셋 아이콘, BETA 태그) | Done | #17, #29 |
| develop 브랜치 생성 | Done | — |
| develop → main 배포 | Done | #21, #24, #31 |
| claude 브랜치 정리 | **미완료** | #38 |

---

## 브랜치 현황

| 브랜치 | 용도 | 상태 |
|--------|------|------|
| `main` | 릴리즈 전용 (보호) | 최신 |
| `develop` | 통합 브랜치 | 최신 |
| `edie-runner` | Runner 배포 소스 | **동기화 필요** |
| `edie-reverse` | Reverse 배포 소스 | 최신 |
| `edie-yut` | Yut Nori 배포 소스 | 최신 |
| `docs/dev-status` | 개발 문서 | 이 문서 |

---

## 미해결 이슈 목록

| # | 제목 | 라벨 |
|---|------|------|
| #32 | [YUT] M4: 보드/말 에디 에셋 교체 | enhancement |
| #33 | [YUT] M5: SFX + 윷 던지기 애니메이션 | enhancement |
| #34 | [YUT] M6: AI 상대 | enhancement |
| #35 | [YUT] M7: 모바일 UX 최적화 | enhancement |
| #36 | [REVERSE] 이스터에그 로고 클릭 동작 확인 | bug |
| #37 | [RUNNER] edie-runner 브랜치 동기화 | bug |
| #38 | [INFRA] claude 관련 브랜치 정리 | chore |
| #39 | [YUT] M8: 온라인 멀티플레이 | enhancement |

---

## 기술 메모

### Crossfire 패턴 수학적 분석
- EDIE 히트박스: y=350..370 (28px 폭)
- 바이러스 히트박스: y+12..y+36 (24px)
- Lane 320: 히트박스 332..356 → EDIE와 겹침 (350..356)
- Lane 340: 히트박스 352..376 → EDIE와 겹침 (352..370)
- 2발 간격 76px > EDIE 28px → 능동적 조작으로 회피 가능
- retain 범위: -200..1480 (2발 모두 생존)

### 윷놀이 보드 구조
- 외곽 20칸 (0~19, 반시계 방향)
- 대각선 A: 5→20→21→24(중앙)→27→28→EXIT
- 대각선 B: 10→22→23→24(중앙)→25→26→EXIT
- 중앙(24)에서는 항상 EXIT 방향(25→26)으로 진행
- 98개 단위 테스트 통과

### AeiROBOT 테마 색상
- 주황: #E8923C (EDIE)
- 초록: #5BE3A8 (AeiROBOT 보조)
- 보라: #9B6FD4 (Amy)
- 분홍: #F28DB4 (AliceM1)
- 빨강: #E64D59 (Alice3)
- 시안: #4FC3F7 (Alice4)
