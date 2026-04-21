Build and verify the Go binaries before tagging a release:

```sh
gofmt -w ./cmd ./internal
go test ./...
go build ./...
git tag 0.1.6-dev # Tag the new version baby
```
