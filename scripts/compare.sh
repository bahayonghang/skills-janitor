#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor compare
set -euo pipefail
exec skills-janitor compare "$@"
