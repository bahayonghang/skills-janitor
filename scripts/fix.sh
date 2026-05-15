#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope fix
set -euo pipefail
exec skillscope fix "$@"
