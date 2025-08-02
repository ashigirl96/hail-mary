# テスト設計ガイドライン

このドキュメントは、hail-maryプロジェクトにおけるテストの設計と実装ガイドラインです。t-wada（和田卓人）さんのテスト設計思想とGoのベストプラクティスを基にしています。

## 🎯 テストの基本思想

### テストコードもプロダクションコード
- **可読性**: テストコードは「仕様書」として機能する
- **保守性**: リファクタリングに耐えうる設計
- **信頼性**: テストが失敗したら必ず問題がある状態を保つ

### テストの価値
```
テストの価値 = 発見できるバグの重要度 × 発見確率 - テストのコスト
```

- 高価値: ビジネスクリティカルな機能の境界値テスト
- 低価値: trivialなgetterのテスト
- 負の価値: false positiveを頻繁に出すテスト

## 📋 テスト分類と戦略

### 1. Unit Tests（単体テスト）
**目的**: 個別のユニットの動作確認  
**特徴**: 高速、独立、決定論的

```go
// Good: 依存関係をモック化
func TestExecutorValidation(t *testing.T) {
    executor := NewExecutor()
    
    tests := []struct {
        name    string
        prompt  string
        wantErr bool
    }{
        {"valid prompt", "Create a function", false},
        {"empty prompt", "", true},
        {"too long prompt", strings.Repeat("a", 10001), true},
    }
    
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            err := executor.validatePrompt(tt.prompt)
            if (err != nil) != tt.wantErr {
                t.Errorf("validatePrompt() error = %v, wantErr %v", err, tt.wantErr)
            }
        })
    }
}
```

### 2. Integration Tests（統合テスト）
**目的**: システム間の連携確認  
**特徴**: 実際の外部依存関係を使用

```go
//go:build integration

func TestClaudeExecutorIntegration(t *testing.T) {
    if testing.Short() {
        t.Skip("Skipping integration test in short mode")
    }
    
    executor := claude.NewExecutor()
    result, err := executor.ExecuteWithSessionTracking("Hello")
    require.NoError(t, err)
    assert.NotEmpty(t, result.ID)
}
```

### 3. Contract Tests（契約テスト）
**目的**: インターフェース契約の確認

```go
func TestExecutorImplementsInterface(t *testing.T) {
    var _ claude.Executor = (*claude.ExecutorImpl)(nil)
    var _ claude.Executor = (*mocks.Executor)(nil)
}
```

## 🎭 モックとスタブの戦略

### 問題: 外部プロセス依存
現在のプロジェクトで発見された問題：テスト中に実際のclaudeプロセスが起動してゾンビプロセスになる。

### 解決策: Interface + Mock Pattern

#### 1. Mock Executor実装
```go
// internal/testing/mocks/executor.go
type Executor struct {
    // 予想される戻り値を設定
    SessionResult     *SessionInfo
    ExecuteError      error
    InteractiveResult error
    
    // 呼び出し履歴の記録
    CallLog []MethodCall
}

type MethodCall struct {
    Method string
    Args   []interface{}
    Time   time.Time
}

func (m *Executor) ExecuteWithSessionTracking(prompt string) (*claude.SessionInfo, error) {
    m.recordCall("ExecuteWithSessionTracking", prompt)
    
    if m.ExecuteError != nil {
        return nil, m.ExecuteError
    }
    return m.SessionResult, nil
}

func (m *MockExecutor) recordCall(method string, args ...interface{}) {
    m.CallLog = append(m.CallLog, MethodCall{
        Method: method,
        Args:   args,
        Time:   time.Now(),
    })
}
```

#### 2. 依存性注入による切り替え
```go
// cmd/prd_init.go
type PRDInitializer struct {
    executor claude.Executor
    logger   *slog.Logger
}

func NewPRDInitializer(executor claude.Executor, logger *slog.Logger) *PRDInitializer {
    return &PRDInitializer{
        executor: executor,
        logger:   logger,
    }
}

// テストでは
func TestPRDInitialization(t *testing.T) {
    mockExecutor := mocks.NewExecutor()
    mockExecutor.SessionResult = &claude.SessionInfo{
        ID:     "test-session",
        Result: "PRD created successfully",
    }
    
    initializer := NewPRDInitializer(mockExecutor, testLogger)
    err := initializer.InitializePRD(context.Background())
    
    require.NoError(t, err)
    assert.Len(t, mockExecutor.CallLog, 1)
    assert.Equal(t, "ExecuteWithSessionTracking", mockExecutor.CallLog[0].Method)
}
```

## 🏗️ テスト構造パターン

### AAA Pattern (Arrange-Act-Assert)
```go
func TestSessionStateManager(t *testing.T) {
    // Arrange
    tempDir := t.TempDir()
    manager := session.NewStateManager(tempDir)
    expectedState := &session.State{
        SessionID: "test-123",
        StartedAt: time.Now(),
    }
    
    // Act
    err := manager.SaveState(expectedState)
    actualState, err2 := manager.LoadState("test-123")
    
    // Assert
    require.NoError(t, err)
    require.NoError(t, err2)
    assert.Equal(t, expectedState.SessionID, actualState.SessionID)
}
```

### Table-Driven Tests
```go
func TestValidateSessionID(t *testing.T) {
    tests := []struct {
        name      string
        sessionID string
        wantErr   bool
        errMsg    string
    }{
        {
            name:      "valid session ID",
            sessionID: "session-123",
            wantErr:   false,
        },
        {
            name:      "empty session ID",
            sessionID: "",
            wantErr:   true,
            errMsg:    "session ID cannot be empty",
        },
        {
            name:      "too short session ID",
            sessionID: "short",
            wantErr:   true,
            errMsg:    "session ID length",
        },
    }
    
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            err := validateSessionID(tt.sessionID)
            
            if tt.wantErr {
                require.Error(t, err)
                assert.Contains(t, err.Error(), tt.errMsg)
            } else {
                require.NoError(t, err)
            }
        })
    }
}
```

## 🎨 テストの可読性ガイドライン

### 1. テスト名は仕様を表現
```go
// Good: 何をテストしているか明確
func TestExecutor_RejectsEmptyPrompt(t *testing.T) {}
func TestSessionManager_SavesStateToCorrectPath(t *testing.T) {}

// Bad: 実装詳細に依存
func TestExecutorValidatePromptFunction(t *testing.T) {}
```

### 2. Contextual Helpers
```go
// テスト用のヘルパー関数
func createTestExecutor(t *testing.T) *mocks.Executor {
    t.Helper()
    executor := mocks.NewExecutor()
    executor.SessionResult = &claude.SessionInfo{
        ID:     "test-session",
        Result: "success",
    }
    return executor
}

func createTempSessionFile(t *testing.T, sessionID string) string {
    t.Helper()
    tempDir := t.TempDir()
    sessionPath := filepath.Join(tempDir, sessionID+".json")
    
    state := &session.State{
        SessionID: sessionID,
        StartedAt: time.Now(),
    }
    
    data, _ := json.Marshal(state)
    os.WriteFile(sessionPath, data, 0644)
    
    return sessionPath
}
```

### 3. エラーメッセージは具体的に
```go
// Good
assert.Equal(t, "expected-session-id", actual.SessionID, 
    "SessionID should match the value passed to ExecuteWithSessionTracking")

// Bad
assert.Equal(t, "expected-session-id", actual.SessionID)
```

## 🚀 CI/CD での実行戦略

### Build Tags活用
```bash
# 通常のテスト実行（unit testのみ）
go test ./...

# 統合テストも含めて実行
go test -tags=integration ./...

# 短時間実行モード
go test -short ./...
```

### Makefile例
```makefile
.PHONY: test test-unit test-integration test-all

test: test-unit

test-unit:
	go test -short ./...

test-integration:
	go test -tags=integration ./...

test-all: test-unit test-integration

test-coverage:
	go test -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html
```

### GitHub Actions設定例
```yaml
jobs:
  test:
    steps:
      - name: Run unit tests
        run: make test-unit
        
      - name: Run integration tests
        run: make test-integration
        if: github.event_name == 'push' && github.ref == 'refs/heads/main'
```

## 🔧 現在のプロジェクト向けリファクタリング計画

### Phase 1: Mock基盤構築 ✅
1. `internal/testing/mocks/executor.go` 作成完了
2. `mocks.Executor` 実装とテスト完了

### Phase 2: 既存テスト移行
1. `executor_test.go` のモック化
2. `prd_init_test.go` のモック化
3. バリデーション・設定テストの分離

### Phase 3: 統合テスト整理
1. 実際のclaudeプロセスが必要なテストを `//go:build integration` で分離
2. CI環境での統合テスト実行戦略確立

## 📚 参考資料

- [t-wada: テストコードの書き方](https://speakerdeck.com/twada/how-to-write-good-test-code)
- [Go Testing Best Practices](https://go.dev/doc/tutorial/add-a-test)
- [TestContainers Go Documentation](https://golang.testcontainers.org/)
- [The Art of Unit Testing](https://www.manning.com/books/the-art-of-unit-testing-second-edition)

## 🎯 品質ゲート

### 必須要件
- [ ] すべてのテストが決定論的
- [ ] テスト実行時間 < 30秒（unit tests）
- [ ] テストカバレッジ > 80%（重要な機能）
- [ ] False positive率 < 1%

### 推奨要件
- [ ] テスト名が仕様を表現
- [ ] 外部依存はモック化
- [ ] 統合テストは分離実行
- [ ] テストコードもコードレビュー対象