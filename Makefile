CARGO ?= cargo
ARGS ?=

.PHONY: all build release run test check clean fmt

all: build

build:
	$(CARGO) build

release:
	$(CARGO) build --release

run:
	$(CARGO) run -- $(ARGS)

test:
	$(CARGO) test

check:
	$(CARGO) check

clean:
	$(CARGO) clean

fmt:
	$(CARGO) fmt
