#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope scan --json
set -euo pipefail
exec skillscope scan --json "$@"
