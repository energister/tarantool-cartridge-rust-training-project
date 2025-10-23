.PHONY: build, setup, dev, test

build:  ## Build the project
	cartridge build
	cargo build

dev: setup  ## Shortcut for setting up the dev environment

setup:  ## Set up the development environment
	./deps.sh

test: build  ## Runs the test suite
	LUA_CPATH=../../target/debug/?.so .rocks/bin/luatest -v