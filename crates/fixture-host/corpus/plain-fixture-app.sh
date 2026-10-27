# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL-2.0/.

# The synthetic plain-fixture app (the miniapp pattern, miniature). A
# REAL consumer of the plain arm: it speaks the runner command contract
# on stdout (capabilities | reset_fixture | observe), keeps a state file,
# and derives every observed node deterministically from its fixture —
# fully synthetic values (no private data, no wall-clock, no network).
# final close: the items array, then the observation object.
# The planted-defect selftest mutates COPIES of this script (declared
# plants: item-drop, initial-drift); this file is never edited by a run.

set -u
STATE_DIR="${FIXTURE_STATE_DIR:-/tmp/fixture-miniapp-state}"
STATE_FILE="${STATE_DIR}/state.env"

# Declared initial fixture (mirrors counterpart manifest fixtures verbatim).
SEED_INITIAL=7
DATASET="synthetic-t3-buttons"

# The observed sections — item order IS the observation order (dash has
# no arrays; the fixed three-section order is intentional for this corpus).
first="1"
print_section() {
    if [ "$first" = "1" ]; then
        first="0"
    else
        printf ','
    fi
}

init_state() {
    mkdir -p "$STATE_DIR"
    printf 'seed=%s\ndataset=%s\n' "$SEED_INITIAL" "$DATASET" > "$STATE_FILE"
}

case "$1" in
    capabilities)
        printf '{"protocol":1,"verbs":["capabilities","reset_fixture","observe"]}\n'
        ;;
    reset_fixture)
        rm -f "$STATE_FILE"
        exit 0
        ;;
    observe)
        if [ ! -f "$STATE_FILE" ]; then
            init_state
        fi
        . "$STATE_FILE"
        # section flags: the item order the app DECLARES (dropping one in a
        # candidate launcher is the item-loss plant the selftest drives).
        include_buttons=1
        include_inputs=1
        include_badges=1
        # section buttons: a widget count derived deterministically from seed
        button_count=$((seed + 1))
        # section inputs: three editable states, one deliberately empty
        input_empty="on"
        # section badges: tone table from the seed
        badges_tone="good/warning/critical"
        printf '{"fixture":{"seed":%s,"dataset":"%s"},"items":[' "$seed" "$DATASET"
        first="1"
        if [ "$include_buttons" = "1" ]; then
            print_section
            printf '{"kind":"section-buttons","node":{"title":"Buttons under seed %s","count":%s,"states":["default","primary","disabled"],"sources":["synthetic-corpus:t3-miniapp/buttons"]}}' "$seed" "$button_count"
        fi
        if [ "$include_inputs" = "1" ]; then
            print_section
            printf '{"kind":"section-inputs","node":{"title":"Inputs","fields":[{"field":"name","state":"empty"},{"field":"email","state":"error"}],"empty_state":"%s","sources":["synthetic-corpus:t3-miniapp/inputs"]}}' "$input_empty"
        fi
        if [ "$include_badges" = "1" ]; then
            print_section
            printf '{"kind":"section-badges","node":{"title":"Badges","tones":"%s","entries":[%s,%s,%s],"sources":["synthetic-corpus:t3-miniapp/badges"]}}' "$badges_tone" $((seed)) $((seed % 5)) $((seed * 2))
        fi
        printf ']}\n'
        ;;
    *)
        printf 'usage: fixture-miniapp.sh {capabilities|reset_fixture|observe}\n' >&2
        exit 2
        ;;
esac
