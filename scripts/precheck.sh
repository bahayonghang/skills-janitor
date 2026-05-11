#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor precheck
set -euo pipefail
exec skills-janitor precheck "$@"
