
export ZKWASMCLI=/home/frank/projects/zkWasm/target/release/delphinus-cli
GUEST="fibonacci-guest"
INPUT=10
K=18

all: run

build:
	cargo build --release --features cuda

subfolder:
	cd ${GUEST} && $(MAKE)

prepare:
	rm -rf params output
	${ZKWASMCLI} --host standard -k ${K} --function zkmain --param ./params --output ./output --wasm ${GUEST}/target/guest.wasm setup

prove:
	${ZKWASMCLI} --host default -k ${K} --function zkmain --output ./output --param ./params --wasm ${GUEST}/target/guest.wasm single-prove --public ${INPUT}:i64 

verify:
	${ZKWASMCLI} --host default -k ${K} --function zkmain --output ./output --param ./params --wasm ${GUEST}/target/guest.wasm single-verify

run:
	RUST_BACKTRACE=1 cargo run --release

clean:
	cd ${GUEST} && $(MAKE) clean
	rm -rf params output target
	rm -f *table.json
