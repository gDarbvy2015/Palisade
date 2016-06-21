package lint

import (
	"encoding/json"
	"strings"
	"testing"
)

func hasCode(r *Report, code string) bool {
