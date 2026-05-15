#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope usage
set -euo pipefail
exec skillscope usage "$@"
