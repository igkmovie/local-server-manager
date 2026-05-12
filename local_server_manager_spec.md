# Local Server Manager 仕様書

## 1. 概要

### 1.1 アプリ名

仮称：**Local Server Manager**

ローカルPC上で動かす複数の開発用サーバ、AIツール用サーバ、MCPサーバ、フロントエンド、API、DB、Workerなどを一元管理するWindowsデスクトップアプリ。

最初はWindowsアプリとして開発する。将来的にmacOSにも対応できるよう、クロスプラットフォーム化しやすい構成にする。

---

## 2. 目的

ローカルで動かすサーバが増えたときに、以下をGUIから管理できるようにする。

- どのサーバが登録されているか確認する
- どのサーバが起動中か確認する
- 登録済みサーバを起動する
- 登録済みサーバを停止する
- 登録済みサーバを再起動する
- ログを確認する
- ポート競合を確認する
- サーバのURLをブラウザで開く
- 複数サーバをグループとして一括起動・一括停止する

---

## 3. 開発方針

### 3.1 初期対象OS

- Windows 10 / Windows 11

### 3.2 将来対象OS

- macOS

### 3.3 推奨技術スタック

- デスクトップアプリ基盤：Tauri v2
- フロントエンド：React + TypeScript
- UI：Tailwind CSS
- バックエンド：Rust
- 設定保存：JSONファイル
- ログ保存：ローカルファイル

### 3.4 なぜTauriか

- Windowsアプリとして配布しやすい
- Electronより軽量
- 将来的にmacOS対応しやすい
- Rust側でプロセス管理、ポート確認、ログ処理を実装しやすい
- フロントエンドはReactで柔軟に作れる

---

## 4. MVPスコープ

最初のバージョンでは以下を実装する。

### 4.1 実装する機能

- サーバ一覧表示
- サーバ登録情報の読み込み
- サーバ起動
- サーバ停止
- サーバ再起動
- サーバ状態表示
- PID管理
- ポート使用状況確認
- HTTPヘルスチェック
- stdout / stderrログ表示
- ログファイル保存
- サーバURLをブラウザで開く
- グループ一括起動
- グループ一括停止

### 4.2 MVPでは実装しない機能

- クラウド同期
- ユーザー認証
- Windows Service化
- Docker管理
- 自動アップデート
- 外部プロセスの強制管理
- 複雑なスケジューラ
- プラグイン機構

---

## 5. 重要な設計方針

### 5.1 基本は「このアプリから起動したサーバ」を管理する

初期版では、原則としてこのアプリから起動したサーバのみを停止・再起動対象にする。

理由：

- 外部から起動されたプロセスは安全に止めてよいか判断しにくい
- 同じポートを別アプリが使っている可能性がある
- プロセスツリーを誤ってkillすると危険
- DBや生成処理中のサーバを強制終了するとデータ破損の可能性がある

外部で同じポートが使われている場合は、以下のように表示する。

```txt
Port 8188 is already in use by another process.
This server may already be running outside Local Server Manager.
```

初期版では外部起動プロセスに対してStopボタンは出さない、または無効化する。

---

## 6. ユースケース

### 6.1 単体サーバを起動する

1. ユーザーがサーバ一覧から対象サーバを選択する
2. Startボタンを押す
3. アプリが設定されたコマンドを指定cwdで起動する
4. PIDを保持する
5. stdout / stderrをログに流す
6. ポートチェックを行う
7. healthUrlがある場合はHTTPチェックを行う
8. running または unhealthy として表示する

### 6.2 単体サーバを停止する

1. ユーザーがStopボタンを押す
2. アプリが対象PIDに対して正常終了を試みる
3. 一定時間待つ
4. 終了しない場合はプロセスツリーごと強制終了する
5. 状態をstoppedにする

### 6.3 サーバを再起動する

1. Stop処理を実行する
2. stoppedになったことを確認する
3. Start処理を実行する

### 6.4 グループを一括起動する

1. ユーザーがグループを選択する
2. Start Groupを押す
3. グループ内サーバをorder順に起動する
4. dependsOnがある場合は依存先がrunningになるまで待つ
5. 失敗した場合は以降の起動を止める
6. エラー内容を表示する

### 6.5 グループを一括停止する

1. ユーザーがグループを選択する
2. Stop Groupを押す
3. 起動順とは逆順で停止する
4. 各サーバの停止結果を表示する

---

## 7. 画面仕様

## 7.1 メイン画面

### レイアウト

```txt
+-------------------------------------------------------------+
| Local Server Manager                         [Settings]      |
+----------------------+--------------------------------------+
| Server List          | Detail Panel                          |
|                      |                                      |
| ● ComfyUI            | Name: ComfyUI                        |
| ● CreaRise API       | Status: Running                      |
| ● Frontend           | Port: 8188                           |
| ● MCP Server         | URL: http://127.0.0.1:8188           |
|                      |                                      |
| Groups               | [Start] [Stop] [Restart] [Open URL]  |
| - CreaRise Dev       |                                      |
| - AI Tools           | Logs                                 |
|                      | ------------------------------------ |
|                      | stdout/stderr log                    |
+----------------------+--------------------------------------+
```

### 左ペイン：サーバ一覧

表示項目：

- ステータスアイコン
- サーバ名
- ポート番号
- 簡易状態

例：

```txt
● ComfyUI        running    :8188
● CreaRise API   stopped    :3001
● Frontend       running    :5173
● MCP Server     error      :7331
```

ステータス色：

- running：緑
- starting：青
- stopped：グレー
- unhealthy：黄
- error：赤
- external：紫

### 中央・右ペイン：詳細

表示項目：

- name
- status
- command
- args
- cwd
- pid
- port
- healthUrl
- openUrl
- lastStartedAt
- lastStoppedAt
- lastExitCode
- lastError

### 操作ボタン

- Start
- Stop
- Restart
- Open URL
- Edit
- Reveal Log

---

## 8. サーバ状態定義

```ts
type ServerStatus =
  | "stopped"
  | "starting"
  | "running"
  | "unhealthy"
  | "stopping"
  | "error"
  | "external";
```

### 8.1 stopped

起動していない状態。

条件：

- このアプリが保持しているPIDがない
- またはPIDのプロセスが存在しない
- 対象ポートも使われていない

### 8.2 starting

起動処理中。

条件：

- プロセス起動直後
- まだhealth checkが成功していない

### 8.3 running

正常起動中。

条件：

- PIDが存在する
- portがある場合、LISTENしている
- healthUrlがある場合、HTTP 200系または許可されたstatus codeを返す

### 8.4 unhealthy

プロセスはいるが正常応答していない。

条件例：

- PIDは存在するがhealthUrlが失敗する
- PIDは存在するがportが開いていない
- 起動タイムアウトを超えた

### 8.5 stopping

停止処理中。

### 8.6 error

起動失敗、異常終了、設定不備など。

### 8.7 external

このアプリから起動したPIDはないが、指定ポートが使用中の状態。

例：

- ComfyUIを手動で起動している
- 別プロセスが同じポートを使用している

---

## 9. 設定ファイル仕様

### 9.1 保存場所

Windows：

```txt
%APPDATA%/LocalServerManager/config.json
```

ログ：

```txt
%APPDATA%/LocalServerManager/logs/
```

将来macOS：

```txt
~/Library/Application Support/LocalServerManager/config.json
~/Library/Application Support/LocalServerManager/logs/
```

---

## 10. config.json仕様

```json
{
  "version": 1,
  "servers": [
    {
      "id": "comfyui",
      "name": "ComfyUI",
      "command": "python",
      "args": ["main.py", "--listen", "127.0.0.1", "--port", "8188"],
      "cwd": "D:/tools/ComfyUI",
      "env": {},
      "port": 8188,
      "healthUrl": "http://127.0.0.1:8188",
      "openUrl": "http://127.0.0.1:8188",
      "autoStart": false,
      "startTimeoutSec": 60,
      "stopTimeoutSec": 10,
      "gracefulStop": true,
      "shell": false,
      "logFile": "comfyui.log"
    }
  ],
  "groups": [
    {
      "id": "ai-tools",
      "name": "AI Tools",
      "servers": [
        {
          "serverId": "comfyui",
          "order": 1,
          "dependsOn": []
        }
      ]
    }
  ]
}
```

---

## 11. サーバ設定スキーマ

```ts
type LocalServerConfig = {
  id: string;
  name: string;
  command: string;
  args: string[];
  cwd: string;
  env?: Record<string, string>;
  port?: number;
  healthUrl?: string;
  openUrl?: string;
  autoStart?: boolean;
  startTimeoutSec?: number;
  stopTimeoutSec?: number;
  gracefulStop?: boolean;
  shell?: boolean;
  logFile?: string;
};
```

### 11.1 id

内部識別子。英数字、ハイフン、アンダースコアのみ。

例：

```txt
comfyui
crearise-api
mcp-x-server
```

### 11.2 name

画面表示名。

### 11.3 command

実行コマンド。

例：

```txt
python
node
npm
pnpm
docker
D:/tools/some-server/start.bat
```

### 11.4 args

コマンド引数。

例：

```json
["main.py", "--port", "8188"]
```

### 11.5 cwd

作業ディレクトリ。

### 11.6 env

環境変数。

例：

```json
{
  "NODE_ENV": "development",
  "PORT": "3001"
}
```

### 11.7 port

状態チェック対象のポート。

### 11.8 healthUrl

HTTPヘルスチェックURL。

指定がある場合は、このURLにGETリクエストを送って状態判定する。

### 11.9 openUrl

Open URLボタンで開くURL。

### 11.10 autoStart

アプリ起動時に自動起動するか。

初期版ではfalse推奨。

### 11.11 startTimeoutSec

起動完了を待つ最大秒数。

デフォルト：60秒

### 11.12 stopTimeoutSec

正常終了を待つ最大秒数。

デフォルト：10秒

### 11.13 gracefulStop

trueの場合、まず正常終了を試す。

### 11.14 shell

trueの場合、シェル経由で起動する。

Windowsでは以下を使う。

```txt
cmd.exe /C
```

ただし、基本はfalse推奨。

---

## 12. グループ設定スキーマ

```ts
type ServerGroupConfig = {
  id: string;
  name: string;
  servers: ServerGroupItem[];
};

type ServerGroupItem = {
  serverId: string;
  order: number;
  dependsOn?: string[];
};
```

例：

```json
{
  "id": "crearise-dev",
  "name": "CreaRise Dev",
  "servers": [
    {
      "serverId": "db",
      "order": 1,
      "dependsOn": []
    },
    {
      "serverId": "api",
      "order": 2,
      "dependsOn": ["db"]
    },
    {
      "serverId": "frontend",
      "order": 3,
      "dependsOn": ["api"]
    }
  ]
}
```

---

## 13. プロセス管理仕様

### 13.1 起動処理

疑似コード：

```ts
async function startServer(serverId: string) {
  const config = getServerConfig(serverId);

  validateConfig(config);

  if (isManagedProcessRunning(serverId)) {
    return { ok: false, reason: "already_running" };
  }

  if (config.port && isPortInUse(config.port)) {
    return { ok: false, reason: "port_in_use_external" };
  }

  const child = spawn(config.command, config.args, {
    cwd: config.cwd,
    env: mergeEnv(config.env),
    shell: config.shell ?? false
  });

  savePid(serverId, child.pid);
  attachLog(serverId, child.stdout, child.stderr);
  setStatus(serverId, "starting");

  const result = await waitForHealthy(serverId, config.startTimeoutSec ?? 60);

  if (result.ok) {
    setStatus(serverId, "running");
  } else {
    setStatus(serverId, "unhealthy");
  }
}
```

### 13.2 停止処理

Windowsでは、最初は安全に停止を試し、失敗したらプロセスツリーを終了する。

疑似コード：

```ts
async function stopServer(serverId: string) {
  const pid = getManagedPid(serverId);

  if (!pid) {
    return { ok: false, reason: "not_managed" };
  }

  setStatus(serverId, "stopping");

  tryGracefulStop(pid);

  const exited = await waitForExit(pid, stopTimeoutSec);

  if (!exited) {
    killProcessTree(pid);
  }

  clearPid(serverId);
  setStatus(serverId, "stopped");
}
```

### 13.3 Windowsでの強制終了

Windowsでは最終手段として以下を実行する。

```txt
taskkill /PID <pid> /T /F
```

`/T` は子プロセスも含める。  
`/F` は強制終了。

### 13.4 注意

DBやファイル書き込み中のサーバは強制終了で破損する可能性がある。  
そのためUIでは、強制終了が発生した場合にログへ記録する。

---

## 14. 状態チェック仕様

### 14.1 状態チェックの優先順位

1. 管理中PIDが存在するか
2. portが設定されている場合、ポートがLISTENしているか
3. healthUrlが設定されている場合、HTTPで正常応答するか

### 14.2 healthUrl判定

初期版では以下を成功扱いにする。

- 200
- 204
- 301
- 302

失敗扱い：

- 接続失敗
- タイムアウト
- 400以上
- DNSエラー

### 14.3 チェック間隔

- 通常時：5秒ごと
- starting中：1秒ごと
- stopping中：1秒ごと

### 14.4 ポート確認

Windowsでは内部実装でTCP接続確認を行う。

- 127.0.0.1:port にTCP接続できれば使用中と判断
- 接続できなければ未使用と判断

---

## 15. ログ仕様

### 15.1 ログの種類

- stdout
- stderr
- system

### 15.2 ログファイル

保存先：

```txt
%APPDATA%/LocalServerManager/logs/{serverId}.log
```

例：

```txt
%APPDATA%/LocalServerManager/logs/comfyui.log
```

### 15.3 ログフォーマット

```txt
[2026-05-12 13:10:23] [stdout] Server started at http://127.0.0.1:8188
[2026-05-12 13:10:24] [stderr] Warning: missing optional dependency
[2026-05-12 13:10:30] [system] Health check passed
```

### 15.4 ログローテーション

初期版では簡易対応。

- 1ファイル最大10MB
- 超えたら `.1` にリネーム
- 古い `.1` があれば上書き

例：

```txt
comfyui.log
comfyui.log.1
```

---

## 16. Tauriコマンド仕様

フロントエンドからRust側へ呼ぶコマンドを定義する。

### 16.1 get_servers

登録済みサーバ一覧を取得する。

```ts
invoke<ServerView[]>("get_servers")
```

戻り値：

```ts
type ServerView = {
  id: string;
  name: string;
  status: ServerStatus;
  port?: number;
  healthUrl?: string;
  openUrl?: string;
  pid?: number;
  lastStartedAt?: string;
  lastStoppedAt?: string;
  lastExitCode?: number;
  lastError?: string;
};
```

### 16.2 start_server

```ts
invoke("start_server", { serverId: "comfyui" })
```

### 16.3 stop_server

```ts
invoke("stop_server", { serverId: "comfyui" })
```

### 16.4 restart_server

```ts
invoke("restart_server", { serverId: "comfyui" })
```

### 16.5 get_server_log

```ts
invoke<string>("get_server_log", {
  serverId: "comfyui",
  lines: 300
})
```

### 16.6 open_server_url

```ts
invoke("open_server_url", { serverId: "comfyui" })
```

### 16.7 start_group

```ts
invoke("start_group", { groupId: "crearise-dev" })
```

### 16.8 stop_group

```ts
invoke("stop_group", { groupId: "crearise-dev" })
```

### 16.9 reload_config

```ts
invoke("reload_config")
```

### 16.10 reveal_config_file

設定ファイルの場所をエクスプローラーで開く。

```ts
invoke("reveal_config_file")
```

---

## 17. フロントエンド状態管理

### 17.1 推奨

- React hooks
- Zustand または Context

初期版ではZustand推奨。

### 17.2 Store例

```ts
type AppStore = {
  servers: ServerView[];
  selectedServerId?: string;
  selectedGroupId?: string;
  logs: Record<string, string>;
  refreshServers: () => Promise<void>;
  startServer: (id: string) => Promise<void>;
  stopServer: (id: string) => Promise<void>;
  restartServer: (id: string) => Promise<void>;
  loadLog: (id: string) => Promise<void>;
};
```

---

## 18. 初期UIコンポーネント

```txt
src/
  App.tsx
  components/
    ServerList.tsx
    ServerDetail.tsx
    ServerActions.tsx
    LogViewer.tsx
    GroupList.tsx
    StatusBadge.tsx
    ConfigErrorBanner.tsx
  stores/
    useAppStore.ts
  types/
    server.ts
```

---

## 19. Rust側モジュール構成

```txt
src-tauri/src/
  main.rs
  commands.rs
  config.rs
  process_manager.rs
  health_checker.rs
  port_checker.rs
  log_manager.rs
  app_state.rs
  models.rs
```

### 19.1 main.rs

Tauri起動とコマンド登録。

### 19.2 commands.rs

Tauri invoke用関数。

### 19.3 config.rs

config.jsonの読み込み・バリデーション。

### 19.4 process_manager.rs

プロセス起動、停止、再起動、PID管理。

### 19.5 health_checker.rs

HTTPヘルスチェック。

### 19.6 port_checker.rs

ポート使用確認。

### 19.7 log_manager.rs

ログファイル出力、読み込み、ローテーション。

### 19.8 app_state.rs

アプリ全体の状態管理。

### 19.9 models.rs

構造体定義。

---

## 20. エラー設計

### 20.1 エラーコード

```ts
type AppErrorCode =
  | "CONFIG_NOT_FOUND"
  | "CONFIG_INVALID"
  | "SERVER_NOT_FOUND"
  | "ALREADY_RUNNING"
  | "NOT_RUNNING"
  | "NOT_MANAGED_PROCESS"
  | "PORT_IN_USE"
  | "COMMAND_NOT_FOUND"
  | "CWD_NOT_FOUND"
  | "START_FAILED"
  | "STOP_FAILED"
  | "HEALTH_CHECK_FAILED"
  | "LOG_READ_FAILED";
```

### 20.2 表示例

#### PORT_IN_USE

```txt
Port 8188 is already in use.
This may mean the server is already running outside this app, or another app is using the same port.
```

#### CWD_NOT_FOUND

```txt
Working directory does not exist:
D:/tools/ComfyUI
```

#### COMMAND_NOT_FOUND

```txt
Command not found:
python

Please check your PATH or use an absolute path.
```

---

## 21. 初期サンプルconfig

```json
{
  "version": 1,
  "servers": [
    {
      "id": "comfyui",
      "name": "ComfyUI",
      "command": "python",
      "args": ["main.py", "--listen", "127.0.0.1", "--port", "8188"],
      "cwd": "D:/tools/ComfyUI",
      "env": {},
      "port": 8188,
      "healthUrl": "http://127.0.0.1:8188",
      "openUrl": "http://127.0.0.1:8188",
      "autoStart": false,
      "startTimeoutSec": 60,
      "stopTimeoutSec": 10,
      "gracefulStop": true,
      "shell": false,
      "logFile": "comfyui.log"
    },
    {
      "id": "crearise-api",
      "name": "CreaRise API",
      "command": "pnpm",
      "args": ["dev"],
      "cwd": "D:/projects/crearise/api",
      "env": {
        "PORT": "3001"
      },
      "port": 3001,
      "healthUrl": "http://127.0.0.1:3001/health",
      "openUrl": "http://127.0.0.1:3001",
      "autoStart": false,
      "startTimeoutSec": 30,
      "stopTimeoutSec": 10,
      "gracefulStop": true,
      "shell": true,
      "logFile": "crearise-api.log"
    },
    {
      "id": "crearise-frontend",
      "name": "CreaRise Frontend",
      "command": "pnpm",
      "args": ["dev", "--host", "127.0.0.1"],
      "cwd": "D:/projects/crearise/frontend",
      "env": {},
      "port": 5173,
      "healthUrl": "http://127.0.0.1:5173",
      "openUrl": "http://127.0.0.1:5173",
      "autoStart": false,
      "startTimeoutSec": 30,
      "stopTimeoutSec": 10,
      "gracefulStop": true,
      "shell": true,
      "logFile": "crearise-frontend.log"
    }
  ],
  "groups": [
    {
      "id": "crearise-dev",
      "name": "CreaRise Dev",
      "servers": [
        {
          "serverId": "crearise-api",
          "order": 1,
          "dependsOn": []
        },
        {
          "serverId": "crearise-frontend",
          "order": 2,
          "dependsOn": ["crearise-api"]
        }
      ]
    },
    {
      "id": "ai-tools",
      "name": "AI Tools",
      "servers": [
        {
          "serverId": "comfyui",
          "order": 1,
          "dependsOn": []
        }
      ]
    }
  ]
}
```

---

## 22. 開発タスク分解

## Phase 1：プロジェクト作成

- Tauri + React + TypeScriptプロジェクト作成
- Tailwind CSS設定
- 基本レイアウト作成
- Tauri invoke疎通確認

完了条件：

- Windowsでアプリが起動する
- React画面が表示される
- Rust側のhelloコマンドを呼べる

---

## Phase 2：config読み込み

- config.json保存場所を決める
- 初回起動時にサンプルconfigを生成
- config.jsonを読み込む
- ServerConfig型を定義
- バリデーション実装
- UIにサーバ一覧を表示

完了条件：

- config.jsonの内容がUIに表示される
- 不正なconfigの場合はエラー表示される

---

## Phase 3：プロセス起動

- start_serverコマンド実装
- cwd存在チェック
- command起動
- args渡し
- env渡し
- PID保存
- stdout/stderr取得
- UIからStartできるようにする

完了条件：

- UIのStartボタンから任意のサーバを起動できる
- 起動したPIDを表示できる

---

## Phase 4：ログ管理

- stdout/stderrをログファイルに保存
- UIでログ表示
- 最新ログ再読み込み
- 簡易ログローテーション

完了条件：

- 起動したサーバのログが画面に表示される
- ログファイルが保存される

---

## Phase 5：状態監視

- PID存在確認
- ポートチェック
- healthUrlチェック
- 定期監視
- ServerStatus更新

完了条件：

- running / stopped / unhealthy / external がUIに表示される
- 外部でポート使用中の場合にexternal表示できる

---

## Phase 6：停止・再起動

- stop_server実装
- graceful stop実装
- timeout後のtaskkill実装
- restart_server実装
- UIボタン接続

完了条件：

- UIからStopできる
- UIからRestartできる
- 子プロセスも含めて終了できる

---

## Phase 7：グループ機能

- group設定読み込み
- start_group実装
- stop_group実装
- order順起動
- 逆順停止
- dependsOn待機

完了条件：

- CreaRise Devなどのグループを一括起動できる
- 一括停止できる

---

## Phase 8：仕上げ

- Open URL実装
- Reveal Config実装
- Reveal Log実装
- UI調整
- エラーメッセージ整備
- Windowsビルド作成

完了条件：

- Windows用アプリとして手元で実用できる

---

## 23. 受け入れ条件

### 23.1 単体サーバ管理

- config.jsonに登録したサーバが一覧に表示される
- Startで起動できる
- Stopで停止できる
- Restartで再起動できる
- 起動中はrunning表示になる
- 停止中はstopped表示になる
- 異常時はerrorまたはunhealthy表示になる

### 23.2 ログ

- stdoutが表示される
- stderrが表示される
- ログがファイルに保存される
- アプリを再起動しても過去ログを確認できる

### 23.3 ポート競合

- 登録ポートが外部で使われている場合、externalまたはport in useとして表示される
- 外部プロセスを誤って停止しない

### 23.4 グループ

- グループ内サーバを順番に起動できる
- 依存関係のあるサーバは依存先の起動後に起動する
- グループ停止は逆順で行う

---

## 24. 実装上の注意

### 24.1 Windowsのnpm / pnpm起動

Windowsでは `pnpm` や `npm` は `.cmd` 経由の場合がある。

そのため、以下のどちらかに対応する。

方法A：shell trueを使う。

```json
{
  "command": "pnpm",
  "args": ["dev"],
  "shell": true
}
```

方法B：実行ファイルの解決を行う。

初期版ではshell trueでよい。

### 24.2 Python仮想環境

Pythonサーバの場合、仮想環境を使うことがある。

例：

```json
{
  "command": "D:/tools/ComfyUI/venv/Scripts/python.exe",
  "args": ["main.py"],
  "cwd": "D:/tools/ComfyUI"
}
```

### 24.3 batファイル起動

batファイルを起動する場合：

```json
{
  "command": "D:/tools/server/start.bat",
  "args": [],
  "cwd": "D:/tools/server",
  "shell": true
}
```

### 24.4 PowerShellは初期版では非推奨

PowerShellは実行ポリシーの問題が出やすいので、初期版ではcmdまたは直接実行を優先する。

---

## 25. セキュリティ方針

このアプリはローカルPC上の任意コマンドを起動できるため、config.jsonは信頼できるユーザーだけが編集する前提にする。

初期版では以下を実施する。

- config.jsonはローカル保存のみ
- 外部からconfigをダウンロードしない
- ネットワーク越しの操作APIを持たない
- アプリ自体はローカルGUI操作のみ

将来的にリモート操作を入れる場合は、認証・CSRF対策・localhost制限が必要。

---

## 26. 将来拡張

### 26.1 GUIでサーバ登録・編集

初期版ではconfig.json編集でもよい。  
後からGUI編集を追加する。

### 26.2 システムトレイ常駐

- 最小化してもトレイに常駐
- サーバ稼働状態をトレイメニューから確認
- 一括起動・停止

### 26.3 自動起動

- アプリ起動時にautoStart=trueのサーバを起動
- Windowsログイン時にアプリを起動

### 26.4 プロファイル切り替え

例：

- AI制作環境
- CreaRise開発環境
- CreaFlow開発環境
- MCP環境

### 26.5 Docker対応

Docker Composeを管理対象にする。

例：

```json
{
  "id": "crearise-docker",
  "name": "CreaRise Docker",
  "type": "docker-compose",
  "command": "docker",
  "args": ["compose", "up"],
  "cwd": "D:/projects/crearise"
}
```

### 26.6 macOS対応

macOSでは停止処理を以下のように切り替える。

- 通常停止：SIGTERM
- 強制停止：SIGKILL
- プロセスツリー停止：pgid管理

---

## 27. Codex / Claude Codeへの開発指示文

以下をそのまま開発AIに渡してよい。

```txt
Tauri v2 + React + TypeScript + Tailwind CSSで、Windows向けのLocal Server Managerアプリを実装してください。

目的：
ローカルPC上で動かす複数の開発用サーバをGUIから管理するアプリです。

MVP機能：
- config.jsonからサーバ一覧を読み込む
- サーバ一覧をUIに表示する
- サーバをStart / Stop / Restartできる
- 起動したプロセスのPIDを管理する
- stdout/stderrをログファイルに保存する
- UIでログを確認できる
- portチェックとhealthUrlチェックで状態表示する
- 外部でポートが使われている場合はexternalとして表示する
- グループ単位で一括起動・一括停止できる
- Open URLボタンでブラウザを開く

重要方針：
- 初期版では、このアプリから起動したPIDのみ停止対象にする
- 外部起動プロセスは勝手にkillしない
- Windows先行。ただし将来macOS対応しやすい構造にする
- config.jsonは %APPDATA%/LocalServerManager/config.json に保存
- ログは %APPDATA%/LocalServerManager/logs/ に保存

実装フェーズ：
1. Tauri + React + TypeScript + Tailwindの初期構築
2. config.json読み込みとサーバ一覧表示
3. start_server実装
4. stdout/stderrログ保存と表示
5. PID / port / healthUrlによる状態監視
6. stop_server / restart_server実装
7. start_group / stop_group実装
8. Open URL / Reveal Config / Reveal Log実装

Rust側モジュール：
- commands.rs
- config.rs
- process_manager.rs
- health_checker.rs
- port_checker.rs
- log_manager.rs
- app_state.rs
- models.rs

React側コンポーネント：
- ServerList.tsx
- ServerDetail.tsx
- ServerActions.tsx
- LogViewer.tsx
- GroupList.tsx
- StatusBadge.tsx
- ConfigErrorBanner.tsx

この仕様書に沿って、まずPhase 1からPhase 3までを実装してください。
```

---

## 28. 最初に作るべき完成形

最初の完成目標は以下。

```txt
アプリを起動する
↓
config.jsonからComfyUI、API、Frontendなどが一覧表示される
↓
Startボタンで起動できる
↓
ログが見える
↓
running / stopped / unhealthy が分かる
↓
Stopで安全に止められる
```

この段階までできれば、実用レベルの土台になる。

---

## 29. 優先順位

### 最優先

1. config読み込み
2. Start
3. Stop
4. ログ表示
5. 状態表示

### 次点

6. Restart
7. Open URL
8. Group起動
9. Group停止

### 後回し

10. GUI編集
11. トレイ常駐
12. 自動起動
13. Docker専用対応
14. macOS対応

---

## 30. 判断メモ

このアプリは、単なるランチャーではなく、ローカル開発環境の制御盤として作る。

ただし初期版で欲張りすぎない。

最初は以下に絞る。

- 登録したものを起動できる
- 起動したものを止められる
- 今どうなっているか分かる
- ログが見える

これだけで十分に価値がある。

