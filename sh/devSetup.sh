#!/bin/sh

cd $(dirname $0)/..
pwd
uv venv --python 3.14
uv pip install --all-extras -r pyproject.toml
echo Activate venv under .venv to develop