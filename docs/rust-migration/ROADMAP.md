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
| 1 | contract 雙 target harness(1b)+ 唯讀公開端點(1a) | ✅ 已完成(`883185e` + `88877c6`);rust contract target 實跑:2 passed + 3 skipped(auth 待 Phase 2) |
| 2 | auth(register/login/profile/refresh/logout + bcrypt→PBKDF2)+ RBAC + rate limiting binding + Google OAuth | ✅ 完成:2a(`df26933`)+ 2b(Google OAuth 手寫 code flow、state HttpOnly cookie、`/api/auth/validate`、PUT profile、linkedin 501 stub);雙 target contract 6/6;`/api/auth/*` 全數移植 |
| 3 | `/api/auth/*` zone route cutover,觀察 48h | 待開始 — 前置:`wrangler secret put JWT_SECRET`、fresh D1 佈建(validate blocker 已於 2b 解除) |
| 4 | events CRUD + approval flow(統一新 schema;管理端點必輸出原始 price_* 欄位) | ✅ 2c 完成(`2fa5002`,2026-08-31):公開詳情 + create/update/delete + approve/reject/publish;rust contract 13/13 |
| 5 | registrations/waitlist(D1 `batch()` 原子重構) | ✅ 2d 完成(2026-08-31):register/cancel/my-registrations + 滿額 waitlist/晉升原子化;rust contract 17/17;stats 兩端同步釘 500 |
| 6+ | participants → sales → analytics(Analytics Engine/KV)→ media/admin | participants 2e、sales CRM 2f、analytics 2g 已完成;media 2h 已實作(2026-08-31):event image/document + venue image upload、event/venue list、owner/admin delete,R2 `MEDIA` binding + D1 metadata + multipart/MIME/10MiB validation;admin 2i 已完成(2026-09-01):`/api/users/*` 七端點 + `/api/admin/database/stats`,requireAdmin/requireSuperAdmin 對齊,backup/restore/cleanup/periodic-backup/checkpoint 維持 501 fallback(鎖定決策 #5) |
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
- **events 管理/詳情偏差(2c,K3 審查記錄)**:(a) 公開詳情可見性 = `status='published' AND approval_status='approved'`(Express 是 `is_active=true`);(b) PUT 接受 camelCase 欄位 — 修正 Express 現行 bug(`EventForm` 送 camelCase 但 Express 的 snake-only whitelist 會 400,即前端更新活動對 Express 本來就是壞的);(c) 詳情路由對 admin 合併了原始欄位與管理行為,但**匿名者看不到 price_*/status** — 比 Express 被遮蔽的管理 handler 更嚴(那個 handler 會把 price_*/status 洩給匿名呼叫者);(d) 非數字 id → rust 404,Express 是 DuckDB cast error 500;(e) update 送陣列/物件到純量欄位 → rust 靜默存 NULL(`to_js` 行為),Express 是 bind error 500 — admin-only、低風險,已留程式碼註解;(f) delete 與 Express 同為硬刪( registrations/waitlist ON DELETE CASCADE 兜底)、requireSuperAdmin、報名數守衛逐行一致
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

- **Phase 3 cutover 前必辦:production secret**:`wrangler secret put JWT_SECRET`(worker 端缺它時所有簽發 token 的端點會 500,與 Express 在 production 啟動即硬失敗等價,是 fail-closed)。`JWT_EXPIRES_IN`(7d/12h/900s/30m/1w,預設 7d)與 `AUTH_RATE_LIMIT_DISABLED=true`(關掉 /api/auth/* 的 rate limiter)是選配 var;contract harness 用的 dummy secret 只寫在 `wrangler.test.toml`
- **analytics cutover 前必辦(2g Stage 2 新增)**:`[vars] CLOUDFLARE_ACCOUNT_ID` + secret `CLOUDFLARE_API_TOKEN`(AE SQL API 讀取),wrangler.toml 已加 `[[analytics_engine_datasets]]` binding `TRACKING`(dataset `hesocial_visitors`);`ANALYTICS_QUERY_STUB=true` 為本機/契約測試的編譯內建 stub 開關
- **media 2h 圖片處理偏差**:R2 key/variant/response/delete 契約保留(`original` + event `thumb/medium`、venue `thumb/medium/large`),但 workers-rs 0.8.5 沒有 typed Images binding,且 Sharp 無法在 isolate 執行;目前 variant 儲存原圖 bytes(可正常顯示但未縮尺寸/轉 JPEG)。若啟用 Cloudflare Images 計費服務,再以 `[images]` binding 取代此暫行路徑;R2 維持 `MEDIA` binding,公開 origin 由 `R2_PUBLIC_URL` 設定。
- **analytics 2g Stage 3 已實作(2026-08-31)**:5 個 D1-backed 分析端點(`events/overview`、`events/performance`、`events/{id}/performance`、`revenue/events`、`engagement/members`)落在 `worker/src/analytics_d1_handlers.rs`,SQL/shaper 在 `core::analytics`「D1-backed analytics (Stage 3)」段;全數維持 admin guard。偏差已文件化於 core 模組文件:舊 DuckDB 欄位映射(`date_time→start_datetime`、`current_attendees→current_registrations`、`pricing_vip→price_platinum`、`pricing_vvip→price_diamond`、`r.tier→r.registration_type`、`categories→event_categories`;Express 的 `events/performance` 對 DuckDB 本是死 500 路徑,移植改為 schema 正確查詢)、`'2025-07'`/`100.0` hardcode **逐字保留**(全時段聚合上的裝飾標籤,parity 優先)、時間謂詞改用 `strftime('%Y-%m-%dT%H:%M:%fZ','now')` 對齊 TEXT 儲存格式。Stage 4 待辦:contract 測試
- **rate limiting 上限無法用 env 配置**:workerd 的 ratelimit binding 的 `simple.period` 只允許 10 或 60 秒(900s 不可行),數值上限寫死在 binding 設定。生產 `wrangler.toml` 用 **2 次/60s**(最壞 30 次/15min,對齊 Express 預設 20/15min 的量級;K3 審查指出原先 100/60 是 ~450x 的爆破預算放寬);契約測試會在同一分鐘打 4+ 次 auth,所以 `wrangler.test.toml` 刻意放寬為 100/60 — 兩檔分離是刻意的。路徑覆蓋整個 `/api/auth/*`(Express 只限 login/register,profile/refresh/logout 在 worker 共用同一 bucket,NAT 後 token-refresh 迴圈可能誤 429)。`AUTH_RATE_LIMIT_DISABLED=true` 是 env 開關;binding 缺失或 `limit()` 出錯時也 fail-open。要 env 化上限得改用 DO/KV 自計數
- **`/api/auth/validate` 已移植(2b)** — 前端 boot 路徑耦合(`useAuth.tsx:46-56` 有 token 就呼叫,非 success 就 logout)已解除;rust 端刻意**不掛 rate limiter**(Express 也沒有,且 boot 時 429 會誤登出)
- **Phase 3 cutover 前必辦(3):D1 佈建假設** — `password_algo` 等新欄位只存在於 `d1/schema.sql` 的 `CREATE TABLE`;cutover 必須對**全新 D1** 套 schema + 資料搬移,不能指向已存在舊表的 D1(否則 SELECT 不存在的欄位,所有登入與認證請求 500)
- **`JWT_EXPIRES_IN` 設了但不合法時 silent fallback 到 7d**(Express 會採用該值)— 可接受的 fail-closed,但設定漂移無人會察覺;接受的拼法:`7d|12h|900s|30m|1w`(不含 `1h30m`/`1.5h`/負數)
- **Rust 驗 JWT 時要求 `iat` 必填**(比 Express 嚴)— jsonwebtoken 預設必發 `iat`,interop fixture 已證實;之後遇到神秘 401 先想到這條(已記於此,不必另查)
- **`/api/auth/*` 已全部移植(2b 後)**:含 `PUT /api/auth/profile`、`GET/POST /api/auth/validate`、`/api/auth/google*`(手寫 code flow)、`/api/auth/linkedin*`(501 stub)
- **pbkdf2 在 worker 內走純 Rust(非 WebCrypto)**:與 host 共用 `core::pbkdf2` 單一實作,wire format 不可能漂移;代價是每次 hash 純 CPU 100k 輪 HMAC-SHA256(登入轉換時是 bcrypt verify + PBKDF2 兩次)。若 Paid plan CPU 觀察到壓力,再評估 WebCrypto `deriveBits` 雙實作 + fixture 互證
- **register/login 的 JSON body 解析**:Rust 端非 JSON body / 非 JSON content-type 由 axum 的 Json rejection 回 400 純文字,Express 是 400 HTML;契約未覆蓋,留下
- **Phase 3 cutover 前必辦(4):`event_participant_access` 的 Rust 寫入路徑(2e 後補審 F1,cutover blocker)** — 該表目前**只有 Express 寫**(`registrationController.ts:147` 註冊建 pending、`:670` 付款更新);`backend-rust` 只讀不寫。註冊切流(2d)且 Express 退役後,新用戶註冊+付款**不會建 epa 列** → `participant_access` 永遠 Unpaid → 所有新註冊的 participants 端點永久 403、名單互不可見。必辦:Rust 端 register(建 pending 列)與付款狀態路徑補寫 epa;同時一併決定 F4 產品決策 — **取消報名不會撤銷已付 epa**(兩端皆然;付款→取消者保留名單可見與 paid 計數,「前任參與者仍在看」案例)。若產品要撤銷,cancel 路徑加 `updateParticipantAccess`;若接受,記錄於此
- **2d/2e 後補審已修(2026-09-01 K3 產出)**:2d 同毫秒晉升競態(晉升子查詢改 `ORDER BY position DESC, id DESC` — 剛接受列必為該 offered_at 中 position 最高,修掉幽靈 +1 與擱淺晉升)、併發重複報名 500→400(batch 錯後重查 existing)、2e detail 端點 LIMIT-1+find 的 404 bug(Express 原樣移植的破 port;改直接 by-id 查詢 + 同可見性謂詞 + 日誌歸屬到正確參與者)。**該宣告未修的移植偏差**:privacy `PUT` 僅布林時的 NOT NULL 衝突已修(COALESCE 子查詢),殘留:檢查順序(deadline→capacity→duplicate→tier→verification vs Rust 順序)與 blocked 字彙('confirmed'/'cancelled' vs 'approved'/'rejected')、fail-open malformed tiers、detail 首列外 404(已修)以外的 404-vs-500 軸(非數字 id Rust 404 vs Express 500)— 契約未釘,留宣言;`js_parse_int` 負數 limit 已 clamp 1..=100;privacyLevel 已加整數檢查
- **exclusivityLevel 過濾在 Rust 端暫不支援**(統一 schema 無對應欄位)— 前端 EventsPage 的級別選擇器送出的參數不會過濾、badges 顯示 null。需要產品決策:映射到 `required_membership_tiers`,或前端改版
- git 歷史清洗(hesocial.duckdb 的 5 個歷史 commit)— 需 force push 決策
- Google OAuth callback 的 state 已改 HttpOnly cookie(2b;passport→手寫 code flow);D1 `users` 的 `password_hash`/`age`/`profession`/`annual_income`/`net_worth` 已放寬為 nullable 以承接 OAuth 建檔(CHECK 對 NULL 放行,非 NULL 仍受約束)— 副作用:register 缺這些欄位時 rust 端會建檔成功(Express 在 NOT NULL 約束下 500),契約未覆蓋,記錄為已知偏差
- **Google OAuth 殘餘差異(2b,K3 審查記錄)**:(a) Google 回 `error=`(如 access_denied)時 rust 轉導 `/login?error=oauth_failed`,Express 是 401 純文字 — rust 的 UX 較好但行為不同;(b) state cookie 的 `Secure` 旗標無條件開啟 — Chrome/Firefox 對 localhost 放行,但 **Safari dev 或任何非 localhost 的 http origin 會靜默丟 cookie,每次 callback 都失敗到 oauth_failed**;(c) `/google` + `/google/callback` 現在掛在 auth rate limiter 內,一次 OAuth 嘗試消耗 2/60s 生產預算,同分鐘重試會吃到裸 429 JSON 而非轉導;(d) validate 端點的 `interests` 已正規化為 JSON 陣列(與 2a 的 login/profile 一致;Express 原樣回傳字串);(e) state 比對非常數時間(128-bit CSPRNG + 600s TTL + 網路傳輸,遠端時序攻擊不可行,K3 判定僅需記錄)
- **registrations/waitlist 偏差(2d,主線深讀記錄;第二 AI 補審待 Kimi 額度歸隊)**:(a) **原子性**:Rust 用 `DB.batch()` + guarded SQL(容量守衛 INSERT+INCREMENT 互鎖、取消→晉升五段 batch 淨計數 -1+1=0),Express 每筆寫入獨立執行、可留半完成狀態 — Rust 較嚴格是刻意改善;(b) **滿額行為**:Express 一律 400 `Event is at full capacity`;Rust 在 `waitlist_enabled=1` 時排入 waitlist(對齊產品資料模型:event_waitlist 表、EventForm 開關、admin waitlist 端點),disabled 時保留 Express 400 — **產品行為變更,可逆,由使用者知悉**;(c) GET/PUT/DELETE /:id 採 owner-or-admin(Express 僅 owner;非 owner 一樣 404,已宣告);(d) `GET /stats/:eventId` 兩端都刻意 500(Express 查不存在的 `event_registrations`,Rust 逐字保留 drift,契約釘死);(e) waitlist position 為 MAX+1 不重編號(排序 token,會有洞);(f) 理論邊界:同毫秒 timestamp 的兩筆併發取消可能在晉升段 double-book(機率極低,Express 完全無原子性更糟,已知悉)
- **sales CRM 偏差(2f,主線實作記錄)**:
  (a) **Express 的 sales 寫入端點本來就壞** — DuckDB 的 `id INTEGER PRIMARY KEY` 沒有 implicit sequence,`createLead`/`createOpportunity`/`createActivity` 的 INSERT 不含 id → `NOT NULL constraint failed: sales_leads.id` → 一律 500。D1 的 `INTEGER PRIMARY KEY` 是 rowid 別名會自動遞增,Rust 真的能建檔。所以三條 create(及依賴它們的斷言)只在 Rust target 開 `salesFlowImplemented` 旗標;read/filter/update/metrics/pipeline/team/delete 用兩邊都有的 fixture 列做**雙 target**。
  (b) **Express 的 leads `search` 與 opportunities `membershipTier`/`assignedTo` 過濾器會 500** — 它的 WHERE 用未限定欄位名,與 JOIN 進來的 `users` 同名列(first_name/last_name/email、membership_tier、assigned_to)在 DuckDB 觸發 `Ambiguous reference`;SQLite 默默取左表,所以 Rust 可用(Rust 一律加 `l.`/`o.`/`a.` 限定)。
  (c) **Express 更新「有子列的 lead/opportunity」會 500** — DuckDB 以 delete-and-reinsert 實作 UPDATE,會撞子表 `sales_opportunities.lead_id`/`sales_activities.opportunity_id` 的外鍵;SQLite 原地更新沒這問題。fixture 因此準備 9004(childless lead)與 9103(childless opportunity)專門跑 update/stage 轉換。
  (d) **deleteLead 的存在檢查是死路徑(釘住的 bug)** — Express 讀 `result.rowCount`,而 DuckDB adapter 只回 `{rows}`,所以恆為 undefined → 刪不到也回 200 `Lead deleted successfully`。Rust 逐字保留(不做存在檢查),契約把幽靈刪除釘成 200。
  (e) **UPDATE 走欄位白名單** — Express 把 `Object.keys(req.body)` 直接拼進 `SET`(任何字串都是可執行的欄位名,是 SQL 注入面)。Rust 改用 `core::sales` 的 snake_case+camelCase 白名單:未知鍵靜默略過(Express 是 500)、`id`/`created_at`/`updated_at` 客戶端不可指定、額外接受前端 `position` → `job_title` 別名;空 body 與 Express 相同(只刷新 updated_at 並回 200)。
  (f) **D1 CHECK 詞彙提前攔截** — `status`/`stage`/`interested_membership_tier`/`membership_tier`/`activity_type` 的合法值與 `lead_score`/`probability` 的 0..100 由 `core::sales::update_value_is_allowed()` 檢查(create 與 update 都走),不符時回與 Express 錯誤路徑相同的 500 信封;DuckDB 沒有這些 CHECK,會照存。NULL 交由欄位自己的 NOT NULL/CHECK 決定(信封相同)。`sales_leads.source` 在 D1 是 NOT NULL、DuckDB 可為 NULL → 缺 source 時 Rust 500、Express 存 NULL。
  (g) **metrics** — SQLite 沒有 `DATE_TRUNC`,改由 `core::sales::period_start()` 以 **UTC** 算出視窗下界再綁參數(Express 用 DB 伺服器時區的 CURRENT_DATE);`salesRepId` 在 Express 是字串直插 SQL(注入面),Rust 綁參數。`period` 不是 monthly/quarterly/yearly 且同時給 `salesRepId` 時,Express 會產生 `FROM sales_leads AND assigned_to = …` 語法錯誤 500,Rust 回 200。funnel 的 conversion/win rate 移到 core 純函式(公式逐字相同,DOUBLE 結果一致);`salesCycleLength: 30` 硬編碼與「月/季/年營收都回同一個 won revenue」的既有行為保留。
  (h) **欄位集合/型別差異** — D1 的 `sales_leads` 多 `last_contact_date`/`next_follow_up_date`,`sales_opportunities` 多 `actual_close_date`/`close_reason`,`sales_pipeline_stages` 多 `color_code`,`sales_activities` 多 `updated_at`(DuckDB 沒有)→ Rust 回應多這些鍵(契約不比對整份 key 集合)。DuckDB 的 BOOLEAN 在 D1 是 INTEGER 0/1,Rust 轉回 `true`/`false` 對齊 Express;DATE/TIMESTAMP 欄位 DuckDB 回 Date 物件(序列化成 ISO 字串)、D1 回原樣 TEXT,所以 `expected_close_date`/`hire_date` 字串格式不同,契約刻意不比較。`interests` 兩邊都是 JSON 字串(Express 送 `JSON.stringify(interests || [])`),Rust 原樣回傳不 parse(前端 `parseInterests` 自己處理)。
  (i) **其它小差異** — `WHERE is_active = true`(DuckDB BOOLEAN)對應 `= 1`(SQLite INTEGER),語意相同;Express 的 `updated_at = CURRENT_TIMESTAMP` 在 Rust 換成 ISO-8601 UTC 的 `now_iso()`(repository 層時間戳規則);Rust 的 lead/opportunity 單筆回應改用 `SELECT *`(對齊 Express 的 `RETURNING *`,不附 join 出來的 `*_first_name`),list/detail 才帶 join 欄。
  (j) **範圍** — 未移植 `sales_targets`/`sales_commissions`(零 runtime 引用,鎖定排除)。Express 也沒有 `GET /sales/opportunities/:id`、activity/pipeline/team 的寫入端點,Rust 同樣不補。sales 沒有掛 worker rate limiter(Express 有通用 `/api` limiter)— 這個缺口與 2c–2e 相同,列在 Phase 3 cutover 前一并處理。
- **participant privacy 偏差(2e,主線深讀記錄)**:(a) view log 在 Express 是逐筆 `await` 且錯誤吞掉(不是 fire-and-forget),但 SQL 使用不存在的 `viewed_participant_id/view_type`;Rust 改用 D1 正式欄位 `participant_id/access_level`,以單一 awaited `DB.batch()` 寫入並同樣吞錯,IP/UA 維持 NULL(Express 雖讀取卻未傳入);(b) D1 `users` 只有 `privacy_level`/`phone_number`,沒有 Express 查詢的 `default_privacy_level`/`phone`/`city`/全域 allow-contact/show-list 欄位,因此 fallback 採 `privacy_level` + per-event boolean 預設 true,`city` 不輸出;(c) access gate 刻意逐行保留:只有 `event_participant_access.payment_status='paid'` 決定能否看,不額外強制 `has_access` 或 registration status;Platinum 最多 level 3,Diamond/Black Card 最多 level 5 且可看聯絡資料;(d) Express participant list 目前還會因 DuckDB result wrapper 誤用 `.map`(應為 `.rows.map`)而 500,且 temp schema 缺上述 user 欄及 `registration_id`;Rust 實作可用 intended flow,所以 participant contract 僅 Rust 開旗標;(e) detail route 保留 Express 的 `page=1,limit=1` 後再 find quirk,contact route 保留只檢查 viewer access/message 且不落地的 TODO success stub
