package prd

import "github.com/ashigirl96/hail-mary/internal/ui"

// SessionOptions contains options for session execution
type SessionOptions struct {
	SelectedInput *ui.UserInput
	IsContinue    bool
}
