#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor search / skills-janitor compare
set -euo pipefail
if [[ "${1:-}" == "--compare" ]]; then
  shift
  exec skills-janitor compare "$@"
fi
exec skills-janitor search "$@"
