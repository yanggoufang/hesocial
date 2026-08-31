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
| 0 | backend-rust spike:workspace + `/api/health` parity + HS256/bcrypt 相容性 proof | ✅ 已完成(commit `3b6454b`) |
| 0.5 | `d1/schema.sql` + `d1/seed.sql` + contract test 雙 target harness(express 先回歸綠) | ✅ schema/seed 已落地並實證;contract harness 併入 Phase 1 首項 |
| 1 | contract 雙 target harness(前置:vitest 0.34→4 升級)→ 唯讀公開端點:`/api/health/*`、`GET /api/events`、`/categories`、`/venues` | 待開始 |
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
- 5 張死表不移植:`user_sessions`、`audit_logs`、`oauth_providers`、`financial_verifications`、`event_feedback`;另 `sales_targets`/`sales_commissions`/`user_preferences` 僅存在於 migration 檔(零 runtime 引用)同樣排除;**visitor 三表不存在於 D1**(鎖定決策 #3,Phase 6 直接做在 Analytics Engine/KV)
- D1 不用舊的 `schema_migrations` 機制 — 未來 D1 schema 變更走 wrangler d1 migrations
- public `/api/events` 的已知偏差(2026-08-31 Codex 審查後確認可接受):pricing 為合成 `{vip,vvip,general,currency}` 非舊自由格式 JSON;`amenities`/`privacyGuarantees`/`videoUrl` 為 null;SQLite `LIKE` 對 ASCII 不分大小寫(DuckDB 會分);`/api/health/status` 的 memory 為佔位 0MB、uptime 為 isolate 生命週期(worker 無 Node process 指標);數字參數解析涵蓋 trim/空字串,hex 等冷門 JS `Number()` 形式不支援
- **Phase 4 必辦**:管理端點必須輸出原始 `price_platinum/price_diamond/price_black_card` 欄位 — `EventManagement.tsx:186` 與 `EventForm.tsx:77` 直接依賴
- 前端 `participantService.ts` 相對路徑 `/api` 在 Render rewrite 下是壞的 → Worker 同 origin 後免費修好

## Phase 1 必辦清單(2026-08-30 Kimi 審查產出)

1. **OPTIONS preflight + 全回應 CORS**:worker 目前只掛 `get(...)`,preflight 會吃 405 且 404/500 回應無 CORS headers(Express 是全部回應都帶 + 短路 OPTIONS)。auth 路由上線前必須補,否則瀏覽器帶 Authorization 的請求全數硬失敗
2. **jsonwebtoken fixture 互通測試**:現有 round-trip 測試是自我一致,不是互通證明。要加一支用 Express `jwt.sign` 實際產出的 token(含 `iat`)當 fixture 在 Rust 端驗證
3. **iat + JWT_EXPIRES_IN**:Rust 簽發側補 `iat`;效期從 hardcoded 7 天改成讀 env(Express 可由 `JWT_EXPIRES_IN` 配置)
4. **ApiEnvelope 補 `message`**:Express 成功回應是 `{success, data, message}`(見 authController),envelope shape 現在就定,不要每端點各自發現
5. **registration port 前置**:Express 新密碼是 cost 12(`authController.ts:39`),移植要用 `bcrypt::hash(pw, 12)`;`bcrypt` crate 在 wasm32(js getrandom)上的**雜湊**路徑要先實證(目前只證了 verify)
6. **health routes 對齊**:移除 `/api/health/detailed`(Express 沒有)、實作 `/api/health/status`(uptime/memory);D1 cutover 時 `database: "duckdb"` 與訊息文字必須同步更新,不能留著說謊
7. **rust gates 進 CI**:等 CI 環境有 Rust + wasm32 target,把 `test:rust`/`lint:rust` 接進 `validate:all`(工具鏈缺席會硬失敗,所以現階段先不接)

## 未決事項

- **exclusivityLevel 過濾在 Rust 端暫不支援**(統一 schema 無對應欄位)— 前端 EventsPage 的級別選擇器送出的參數不會過濾、badges 顯示 null。需要產品決策:映射到 `required_membership_tiers`,或前端改版
- git 歷史清洗(hesocial.duckdb 的 5 個歷史 commit)— 需 force push 決策
- Google OAuth callback 的 state 改 HttpOnly cookie(passport→手寫 code flow)
- refresh token 缺陷(重簽同一 payload)cutover 後修
