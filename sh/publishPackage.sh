#!/bin/bash
set -e
cargo package
cargo publish
# git tag 0.1.6
# git push --tags
