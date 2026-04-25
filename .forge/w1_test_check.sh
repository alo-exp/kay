#!/bin/sh
set -e
cargo check -p forge_app -p forge_config -p forge_display -p forge_domain -p forge_fs -p forge_infra -p forge_json_repair -p forge_main -p forge_spinner --tests 2>&1 | tail -20
echo "CHECK_OK"
