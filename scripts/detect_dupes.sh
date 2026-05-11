#!/bin/bash
# Legacy compatibility wrapper. Prefer: skills-janitor report
set -euo pipefail
exec skills-janitor report "$@"
