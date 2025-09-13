#!/bin/bash
[ -f dist/misterio*.tar.gz  ] && rm -r dist
python3 -m build
python3 -m twine upload dist/*
# Tag the new version baby
git tag 0.1.3

