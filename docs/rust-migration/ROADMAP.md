# Rust 遷移 Roadmap(Node/Express/DuckDB → Rust/workers-rs/D1)

> 本文件是遷移的單一真相來源。團隊決策日:2026-08-30。前端維持 React/TS 不變,只遷移後端。

## 目標架構(已定案)

- 平台:Cloudflare Workers(workers-rs + axum,default-features = false)
- 資料庫:Cloudflare D1(取代 DuckDB;解決 edge 上 DuckDB 不持久、每次要從 R2 拷回的問題)
- 媒體:R2(不變)
- Repo 佈局:`backend-rust/` workspace = `crates/core`(host-native domain,`cargo test` 可測)+ `crates/worker`(wasm32 glue)
- API 路徑不變;用 Cloudflare zone route 逐系統切流,可即時回滾
- Rust token 必須與 Express bit-for-bit 互通(HS256,`{userId,email,membershipTier}` + 7d)

## 已鎖定的決策(2026-08-30 使用者拍板)

1. Workers Paid($5/mo)+ 密碼 bcrypt→PBKDF2 lazy rehash(首登時轉換,不強制重設)
2. D1 `events` 直接統一成 event-management 形狀(title/slug/start_datetime),response JSON 逐字不變
3. visitor tracking / analytics → Cloudflare Analytics Engine 或 KV(不進 D1,避開單寫入者瓶頸)
4. 自訂網域可掛 Cloudflare → 走 zone route 灰度 cutover
5. DuckDB 專屬端點(backup/restore/checkpoint/periodic-backup、deployment、emergency)→ 回 501,`BackupManagement.tsx` 下線

## 驗收契約

- `backend/test/` 的 characterization/contract tests 是每個系統的移植門檻:rust target 全綠 + express target 無回歸才能 cutover
- 每階段 gate:`cargo fmt --check`、`cargo clippy --target wasm32-unknown-unknown -- -D warnings`、`cargo test -p core`、`wrangler deploy --dry-run`、`npm run validate:all`

## 階段與狀態

| Phase | 內容 | 狀態 |
|---|---|---|
| 0 | backend-rust spike:workspace + `/api/health` parity + HS256/bcrypt 相容性 proof | ✅ 已完成(commit 中) |
| 0.5 | `d1/schema.sql` + `d1/seed.sql` + contract test 雙 target harness(express 先回歸綠) | ⏳ 進行中 |
| 1 | 唯讀公開端點:`/api/health/*`、`GET /api/events`、`/categories`、`/venues` | 待開始 |
| 2 | auth(register/login/profile/refresh/logout + bcrypt→PBKDF2)+ RBAC + rate limiting binding | 待開始 |
| 3 | `/api/auth/*` zone route cutover,觀察 48h | 待開始 |
| 4 | events CRUD + approval flow(統一新 schema) | 待開始 |
| 5 | registrations/waitlist(D1 `batch()` 原子重構) | 待開始 |
| 6+ | participants → sales → analytics(Analytics Engine/KV)→ media/admin | 待開始 |
| 終 | DuckDB→D1 資料搬移、Render API 下線、`backend/` 歸檔 | 待開始 |

## 已知技術債/陷阱(移植時處理)

- `database/duckdb-schema.sql` 在乾淨 DuckDB 上建不起表(ON DELETE 不支援、欄位重複、INTEGER id vs UUID 混用)— 測試目前用 regex 修補
- D1 無 interactive transaction:`registrationController` 多步寫入要重構成 `batch()`
- 時間戳雙格式會在 SQLite 排序 silently 出錯 → repository 層統一 ISO-8601 UTC
- `jsonwebtoken` crate 依賴 ring(wasm 不可用)→ 純 Rust HMAC-SHA256
- Workers Free 10ms CPU 跑不動 bcrypt cost 12 → 必須 Paid(已決策)
- 5 張死表不移植:`user_sessions`、`audit_logs`、`oauth_providers`、`financial_verifications`、`event_feedback`
- 前端 `participantService.ts` 相對路徑 `/api` 在 Render rewrite 下是壞的 → Worker 同 origin 後免費修好

## 未決事項

- git 歷史清洗(hesocial.duckdb 的 5 個歷史 commit)— 需 force push 決策
- Google OAuth callback 的 state 改 HttpOnly cookie(passport→手寫 code flow)
- refresh token 缺陷(重簽同一 payload)cutover 後修
