
CLI="/home/frank/projects/zkWasm/target/release/delphinus-cli"
GUEST="fibonacci-guest"
INPUT=10

all: build subfolder prepare prove verify

build:
	cargo build --release --features cuda

subfolder:
	cd ${GUEST} && $(MAKE)

prepare:
	rm -rf params output
	${CLI} --host standard -k 18 --function zkmain --param ./params --output ./output --wasm ${GUEST}/target/guest.wasm setup

prove:
	${CLI} --host default -k 18 --function zkmain --output ./output --param ./params --wasm ${GUEST}/target/guest.wasm single-prove --public ${INPUT}:i64 

verify:
	${CLI} --host default -k 18 --function zkmain --output ./output --param ./params --wasm ${GUEST}/target/guest.wasm single-verify

clean:
	cd ${GUEST} && $(MAKE) clean
	rm -rf params output target
	rm -f *table.json
