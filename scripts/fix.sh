#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor fix
set -euo pipefail
exec skills-janitor fix "$@"
