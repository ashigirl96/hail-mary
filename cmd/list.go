package cmd

import (
	"fmt"
	"log/slog"

	"github.com/spf13/cobra"
)

var (
	// listコマンド固有のフラグ
	showAll bool
	format  string
)

var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List items with various options",
	Long:  `List command demonstrates a regular CLI command that doesn't use TUI.`,
	ValidArgsFunction: func(cmd *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
		// 引数の位置でもフラグの候補を返す
		if len(args) == 0 && toComplete == "" {
			// 動的にフラグを取得
			return GetFlagCompletions(cmd), cobra.ShellCompDirectiveNoFileComp
		}

		return nil, cobra.ShellCompDirectiveNoFileComp
	},
	Run: func(cmd *cobra.Command, args []string) {
		logger := GetLogger()

		logger.Debug("Executing list command",
			slog.Bool("show-all", showAll),
			slog.String("format", format),
			slog.Any("args", args),
		)

		// サンプルデータ
		items := []string{
			"item-1: Configuration file",
			"item-2: Database connection",
			"item-3: API endpoint",
		}

		if !showAll {
			items = items[:2] // 最初の2つだけ表示
		}

		switch format {
		case "json":
			fmt.Println("{")
			for i, item := range items {
				fmt.Printf("  \"%d\": \"%s\"", i, item)
				if i < len(items)-1 {
					fmt.Print(",")
				}
				fmt.Println()
			}
			fmt.Println("}")
		case "csv":
			for i, item := range items {
				fmt.Printf("%d,%s\n", i, item)
			}
		default:
			for _, item := range items {
				fmt.Println(item)
			}
		}

		logger.Info("List command completed",
			slog.Int("items_shown", len(items)),
		)
	},
}

func init() {
	rootCmd.AddCommand(listCmd)

	// フラグの設定
	listCmd.Flags().BoolVarP(&showAll, "all", "a", false, "Show all items")
	listCmd.Flags().StringVarP(&format, "format", "f", "text", "Output format (text, json, csv)")

	// formatフラグの補完関数を登録
	_ = listCmd.RegisterFlagCompletionFunc("format", func(cmd *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
		return []string{"text", "json", "csv"}, cobra.ShellCompDirectiveNoFileComp
	})
}
