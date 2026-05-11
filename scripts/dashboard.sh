#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor dashboard --open
set -euo pipefail
exec skills-janitor dashboard --open "$@"
