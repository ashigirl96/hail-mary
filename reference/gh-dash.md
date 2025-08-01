# gh-dash タブ管理とビュー切り替えの実装

## 概要

gh-dashは、GitHub PR、Issue、Repositoryを管理するTUIアプリケーションで、複数のビューとタブを効率的に管理する仕組みを実装しています。

## アーキテクチャ

### 1. ビュー構造

gh-dashは3つのメインビューを持っています：

- **PRsView**: Pull Requestの一覧表示
- **IssuesView**: Issueの一覧表示  
- **RepoView**: Repositoryの一覧表示

### 2. コンポーネント構成

```
ui/
├── ui.go                    # メインのUIモデルとビュー管理
├── components/
│   ├── tabs/               # タブバーコンポーネント
│   │   └── tabs.go
│   ├── carousel/           # サイドバー内タブ切り替え
│   │   └── carousel.go
│   ├── section/            # 各セクションの基底実装
│   │   └── section.go
│   ├── prssection/         # PRセクション
│   ├── issuessection/      # Issueセクション
│   └── reposection/        # Repoセクション
```

## タブ管理の実装

### 1. Tabsコンポーネント (`ui/components/tabs/tabs.go`)

```go
type Model struct {
    sectionsConfigs []config.SectionConfig  // 各セクションの設定
    sectionCounts   []SectionState         // 各セクションの状態（カウント、ローディング）
    CurrSectionId   int                    // 現在選択中のセクションID
}

type SectionState struct {
    Count     int
    IsLoading bool
    spinner   spinner.Model
}
```

**主要メソッド：**
- `View()`: タブバーのレンダリング
- `UpdateSectionsConfigs()`: セクション設定の更新
- `UpdateSectionCounts()`: 各セクションのカウント更新
- `SetCurrSectionId()`: 現在のセクションIDの設定

### 2. レンダリング処理

```go
func (m Model) View(ctx *context.ProgramContext) string {
    var tabs []string
    for i, sectionTitle := range sectionTitles {
        if m.CurrSectionId == i {
            tabs = append(tabs, ctx.Styles.Tabs.ActiveTab.Render(sectionTitle))
        } else {
            tabs = append(tabs, ctx.Styles.Tabs.Tab.Render(sectionTitle))
        }
    }
    
    // タブをパイプ（|）で区切って表示
    renderedTabs := lipgloss.JoinHorizontal(lipgloss.Top, 
        strings.Join(tabs, ctx.Styles.Tabs.TabSeparator.Render("|")))
    
    return ctx.Styles.Tabs.TabsRow.Render(
        lipgloss.JoinHorizontal(lipgloss.Center, renderedTabs, version))
}
```

## ビュー切り替えの実装

### 1. ビュー管理 (`ui/ui.go`)

```go
type Model struct {
    repo          section.Section    // Repoビューのセクション
    prs           []section.Section  // PRビューの複数セクション
    issues        []section.Section  // Issueビューの複数セクション
    tabs          tabs.Model         // タブコンポーネント
    currSectionId int                // 現在のセクションID
    ctx           *context.ProgramContext
}
```

### 2. ビュー切り替えロジック

```go
func (m *Model) switchSelectedView() config.ViewType {
    switch m.ctx.View {
    case config.RepoView:
        return config.PRsView
    case config.PRsView:
        return config.IssuesView
    case config.IssuesView:
        return config.RepoView
    }
}
```

### 3. セクション取得

```go
func (m *Model) getCurrentViewSections() []section.Section {
    if m.ctx.View == config.RepoView {
        return []section.Section{m.repo}
    } else if m.ctx.View == config.PRsView {
        return m.prs
    } else {
        return m.issues
    }
}
```

## Carouselコンポーネント（サイドバー内タブ）

PRサイドバーでは、Carouselコンポーネントを使用して複数のタブを管理：

```go
var tabs = []string{" Overview", " Checks", " Activity", " Files Changed"}

c := carousel.New(
    carousel.WithItems(tabs),
    carousel.WithWidth(ctx.MainContentWidth),
)
```

**Carouselの特徴：**
- 横スクロール可能なタブUI
- キーボードナビゲーション（左右キー）
- 現在の選択状態を視覚的に表示

## キーボードナビゲーション

### 1. ビュー切り替え

```go
// PRビューからの切り替え
case key.Matches(msg, keys.PRKeys.ViewIssues):
    m.ctx.View = m.switchSelectedView()
    m.tabs.UpdateSectionsConfigs(m.ctx)

// Issueビューからの切り替え  
case key.Matches(msg, keys.IssueKeys.ViewPRs):
    m.ctx.View = m.switchSelectedView()
    m.tabs.UpdateSectionsConfigs(m.ctx)
```

### 2. セクション間の移動

- `Tab`: 次のセクションへ
- `Shift+Tab`: 前のセクションへ

### 3. Carousel内のタブ移動

- `h`/`←`: 左のタブへ
- `l`/`→`: 右のタブへ

## 状態管理

### 1. 非同期データ更新

各セクションは独立してデータをフェッチ：

```go
func (m *Model) fetchAllViewSections() ([]section.Section, tea.Cmd) {
    cmds := make([]tea.Cmd, 0)
    cmds = append(cmds, m.tabs.SetAllLoading()...)
    
    // 各セクションのフェッチコマンドを追加
    // ...
    
    return newSections, tea.Batch(cmds...)
}
```

### 2. ローディング状態の表示

```go
if m.sectionCounts[i].IsLoading {
    title = fmt.Sprintf("%s %s", title, m.sectionCounts[i].spinner.View())
} else {
    title = fmt.Sprintf("%s (%s)", title, utils.ShortNumber(m.sectionCounts[i].Count))
}
```

## レイアウト計算

画面サイズに応じて動的にレイアウトを調整：

```go
func (m *Model) onWindowSizeChanged(msg tea.WindowSizeMsg) {
    m.ctx.ScreenWidth = msg.Width
    m.ctx.ScreenHeight = msg.Height
    
    if m.footer.ShowAll {
        m.ctx.MainContentHeight = msg.Height - common.TabsHeight - common.ExpandedHelpHeight
    } else {
        m.ctx.MainContentHeight = msg.Height - common.TabsHeight - common.FooterHeight
    }
    
    m.syncMainContentWidth()
}
```

## 実装の特徴

1. **独立性**: 各ビューとセクションは独立した状態を保持
2. **再利用性**: セクションインターフェースによる共通化
3. **非同期性**: 各セクションが独立してデータを取得
4. **レスポンシブ**: 画面サイズに応じた動的レイアウト
5. **パフォーマンス**: 必要なセクションのみレンダリング

## TUIでのShell管理への応用

gh-dashのアーキテクチャを参考に、2つのshellをTUIで管理する場合：

1. **ビューとして各shellを管理**: 各shellを独立したセクションとして実装
2. **タブバーで切り替え**: 上部にタブバーを配置してshell間を切り替え
3. **状態の独立管理**: 各shellの出力バッファとプロセス状態を分離
4. **非同期I/O**: 各shellからの出力を非同期で処理

ただし、実際のshell実装には追加で以下が必要：
- PTY（擬似端末）の管理
- ANSIエスケープシーケンスの処理
- 入力のルーティング（フォーカスされたshellへ）
- プロセス管理とシグナルハンドリング