.PHONY: build, setup, dev, test

build:  ## Build the project
	cartridge build
	cargo build

dev: setup  ## Shortcut for setting up dev environment

setup:  ## Set up development environment
	./deps.sh

test: build  ## Runs the test suite. Optionally, provide a test name as the second argument to run only that test
	rm -f tmp/tarantool.log
	test_name="$(word 2,$(MAKECMDGOALS))"; \
	(LUA_CPATH=../../target/debug/?.so TARANTOOL_LOG_LEVEL=2 TARANTOOL_LOG=../../tmp/tarantool.log .rocks/bin/luatest -v $$test_name) || (echo "Tarantool log:" && cat tmp/tarantool.log && false)

# This prevents make from trying to execute <test_name> as a second target and make a file named after the test_name
%:
	@: