#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor scan --json
set -euo pipefail
exec skills-janitor scan --json "$@"
