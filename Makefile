PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
BIN ?= ifs
INSTALL_PATH ?= $(BINDIR)/$(BIN)

CARGO ?= cargo
INSTALL ?= install
RM ?= rm -f

ifeq ($(shell id -u),0)
SUDO ?=
else
SUDO ?= sudo
endif

BUILD_BIN := target/release/$(BIN)

.PHONY: all build ensure-install-build install uninstall test fmt clippy check clean

all: build

build:
	$(CARGO) build --release

ensure-install-build:
	@if [ "$$(id -u)" -eq 0 ]; then \
		test -x "$(BUILD_BIN)" || { \
			echo "$(BUILD_BIN) is missing; run 'make build' before 'sudo make install'."; \
			exit 1; \
		}; \
	else \
		$(MAKE) build; \
	fi

install: ensure-install-build
	$(SUDO) $(INSTALL) -d "$(dir $(INSTALL_PATH))"
	$(SUDO) $(INSTALL) -m 0755 "$(BUILD_BIN)" "$(INSTALL_PATH)"

uninstall:
	$(SUDO) $(RM) "$(INSTALL_PATH)"

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt --check

clippy:
	$(CARGO) clippy --all-targets -- -D warnings

check: fmt test clippy

clean:
	$(CARGO) clean
