GO ?= go

.PHONY: build test fmt vet binaries

build:
	$(GO) build ./...

test:
	$(GO) test ./...

fmt:
	$(GO) fmt ./...

vet:
	$(GO) vet ./...

# GG Added to generate binaries
binaries:
	mkdir -p ./bin
	$(GO) build -o ./bin/misterio ./cmd/misterio
	$(GO) build -o ./bin/misterio-add ./cmd/misterio-add
	$(GO) build -o ./bin/misterio-mv ./cmd/misterio-mv
	$(GO) build -o ./bin/misterio-rm ./cmd/misterio-rm
