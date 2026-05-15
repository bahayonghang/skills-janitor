#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope report
set -euo pipefail
exec skillscope report "$@"
