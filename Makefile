SRC = src/bestclient.rs
DEPS_FILE = target/.bestclient.deps
DEPS = $(shell head -n1 $(DEPS_FILE) 2> /dev/null)
BIN = target/bestclient
CFG_OPT ?= -L target/deps -g # -O

all: $(BIN)

$(BIN): $(DEPS)
	rustc $(CFG_OPT) --out-dir target $(SRC)
	@rustc $(CFG_OPT) --no-trans --dep-info $(DEPS_FILE) $(SRC) 2> /dev/null
	@sed -i 's/.*: //' $(DEPS_FILE)

doc:
	rm -rf doc
	rustdoc $(CFG_OPT) $(SRC)

clean:
	rm -rf $(BIN) doc

.PHONY: all clean doc
