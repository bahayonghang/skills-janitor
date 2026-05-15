#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope tokens
set -euo pipefail
exec skillscope tokens "$@"
