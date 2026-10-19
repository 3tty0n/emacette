PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
CARGO ?= cargo
PYTHON ?= python3
BIN := target/release/nemacs

.PHONY: all build test check fmt e2e bench install uninstall clean release

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

e2e: build
	$(PYTHON) scripts/e2e.py $(BIN)

bench: build
	$(PYTHON) scripts/bench.py $(BIN)

release:
	./scripts/release.sh $(ARGS)

install: build
	mkdir -p $(DESTDIR)$(BINDIR)
	install -m 755 $(BIN) $(DESTDIR)$(BINDIR)/nemacs

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/nemacs

clean:
	$(CARGO) clean
