#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope dashboard --open
set -euo pipefail
exec skillscope dashboard --open "$@"
