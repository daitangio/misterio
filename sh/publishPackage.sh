#!/bin/bash
set -e
gofmt -w ./cmd ./internal
go test ./...
go build ./...
# Tag the new version baby
# git tag 0.1.6-dev
#git push --tags
