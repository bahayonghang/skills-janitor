#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor tokens
set -euo pipefail
exec skills-janitor tokens "$@"
