# vim:sts=8 sw=8 ts=8 noexpandtab:
#
#   Makefile
#   ~~~~~~~~
#
#   Project:		cubus
#
#   Text encoding:      UTF-8
#
#   Created 2023-07-09:	Ulrich Singer
#


# What to install
PROGRAM		= cubus


## Derived Paths ##

PRGREL		= ./target/release/$(PROGRAM)


## Targets ##

.PHONY: all
all: program

.PHONY: program
program: $(PRGREL)

$(PRGREL): ./src/main.rs
	cargo build --release

tags: ./src/main.rs
	echo $^ | xargs rstags

.PHONY: install
install: $(PRGREL)
	cargo install --path .

.PHONY: clean
clean:
	cargo clean


# ~ Makefile ~ #
