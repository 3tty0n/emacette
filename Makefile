PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
CARGO ?= cargo
PYTHON ?= python3
BIN := target/release/nemacs

.PHONY: all build test check fmt smoke bench install uninstall clean

all: build

build:
	$(CARGO) build --release

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt

check:
	$(CARGO) fmt -- --check
	$(CARGO) test

smoke: build
	$(PYTHON) scripts/smoke.py $(BIN)

bench: build
	$(PYTHON) scripts/bench.py $(BIN)

install: build
	install -Dm755 $(BIN) $(DESTDIR)$(BINDIR)/nemacs

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/nemacs

clean:
	$(CARGO) clean
