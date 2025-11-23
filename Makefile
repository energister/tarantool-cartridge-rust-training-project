.PHONY: build, setup, dev, test, start, setup-cluster, load-test

build:  ## Build the project
	cartridge build
	cargo build

dev: setup  ## Shortcut for setting up dev environment

setup:  ## Set up development environment
	./deps.sh

test: build  ## Runs the test suite. Optionally, provide a test name via `test` variable to run only that test
	rm -f tmp/tarantool.log
	# first path is for the router_role.lua, second path is for other roles
	(LUA_CPATH="../../target/debug/?.so;target/debug/?.so" \
	TARANTOOL_LOG_LEVEL=2 \
	TARANTOOL_LOG=../../tmp/tarantool.log \
	.rocks/bin/luatest -v -c $(test) ) || \
	(echo "Tarantool log:" && cat tmp/tarantool.log && false)

start: build
	LUA_CPATH="../../target/debug/?.so;target/debug/?.so" cartridge start -d

setup-cluster:
	cartridge replicasets setup --bootstrap-vshard

load-test:
	k6 run test/k6-test.js