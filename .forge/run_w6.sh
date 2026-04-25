#!/bin/sh
cd "$(dirname "$0")/.."
forge -p "$(cat .forge/w6_prompt.txt)"
