#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope search / skillscope compare
set -euo pipefail
if [[ "${1:-}" == "--compare" ]]; then
  shift
  exec skillscope compare "$@"
fi
exec skillscope search "$@"
