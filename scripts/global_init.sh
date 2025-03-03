#!/usr/bin/env sh

# Run this script from the root directory

./scripts/init_bridge.sh &&
    ./scripts/init_db.sh &&
    ./scripts/build_image.sh &&
    ./scripts/run_container.sh
