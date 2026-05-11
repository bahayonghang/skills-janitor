#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor usage
set -euo pipefail
exec skills-janitor usage "$@"
