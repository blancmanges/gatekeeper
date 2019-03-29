#!/bin/bash -euo pipefail

set +e; {
read -r -d '' MPL_HEADER <<'EOM'
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
EOM

read -r -d '' AWK_SCRIPT <<'EOM'
printed == 3 {
    exit 0
}
/./ {
    print
    printed++
}
EOM
}; set -e

EXITCODE=0

while read -r FILE; do
    FILE_HEADER="$(awk "${AWK_SCRIPT}" "${FILE}")"
    if [[ "${FILE_HEADER}" = "${MPL_HEADER}" ]]; then
        echo "${FILE}  OK"
    else
        echo "${FILE}  FAILED"
        EXITCODE=1
    fi
done <<< "$(find . -not \( -path './target/*' -o -path './build-cache/*' \) -a -type f -name '*.rs')"

exit "${EXITCODE}"
