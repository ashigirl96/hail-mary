package cmd

import (
	"log/slog"

	"github.com/ashigirl96/hail-mary/internal/ui"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/spf13/cobra"
)

var (
	// UIコマンド固有のフラグ
	initialText string
)

var uiCmd = &cobra.Command{
	Use:   "ui",
	Short: "Launch the interactive TUI",
	Long:  `Launch an interactive Terminal User Interface using Bubbletea.`,
	ValidArgsFunction: func(cmd *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
		// 引数の位置でもフラグの候補を返す
		if len(args) == 0 && toComplete == "" {
			// 動的にフラグを取得
			return GetFlagCompletions(cmd), cobra.ShellCompDirectiveNoFileComp
		}

		return nil, cobra.ShellCompDirectiveNoFileComp
	},
	RunE: func(cmd *cobra.Command, args []string) error {
		logger := GetLogger()

		logger.Info("Launching TUI",
			slog.String("initial_text", initialText),
		)

		// TUIモデルの初期化
		model := ui.NewModel(initialText, logger)

		// Bubbletea プログラムの作成と実行
		p := tea.NewProgram(model)

		// TUIを実行
		finalModel, err := p.Run()
		if err != nil {
			logger.Error("TUI error", slog.Any("error", err))
			return err
		}

		// 最終的なモデルの状態をログ
		if m, ok := finalModel.(ui.Model); ok {
			logger.Info("TUI closed",
				slog.String("final_input", m.GetInput()),
				slog.Bool("confirmed", m.IsConfirmed()),
			)
		}

		return nil
	},
}

func init() {
	rootCmd.AddCommand(uiCmd)

	// フラグの設定
	uiCmd.Flags().StringVarP(&initialText, "text", "t", "", "Initial text for the input field")
}
