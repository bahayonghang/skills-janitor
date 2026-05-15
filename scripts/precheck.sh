#!/bin/bash
# Legacy compatibility wrapper. Prefer: skillscope precheck
set -euo pipefail
exec skillscope precheck "$@"
