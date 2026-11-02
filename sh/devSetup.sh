#!/bin/sh

cd $(dirname $0)/..
pwd
uv venv --python 3.14
# ? pip install -e ".[test]"
uv pip install --all-extras -r pyproject.toml
echo Activate venv under .venv to develop