package schemas

import (
	"github.com/go-playground/validator/v10"
)

// Validate is a shared validator instance for all schemas
var Validate = validator.New(validator.WithRequiredStructEnabled())
