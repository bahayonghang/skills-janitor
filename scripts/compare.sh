#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope compare
set -euo pipefail
exec skillscope compare "$@"
