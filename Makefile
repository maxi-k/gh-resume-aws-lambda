all: clean build-lambda-release

clean:
	rm -rf target/lambda/release/bootstrap
	rm -f target/lambda-rust.zip

target/lambda/release/bootstrap:
	docker run --rm \
	  -v ${PWD}:/code \
	  -v ${HOME}/.cargo/registry:/root/.cargo/registry \
	  -v ${HOME}/.cargo/git:/root/.cargo/git \
	  softprops/lambda-rust:0.2.7-rust-1.44.1

target/lambda-rust.zip: target/lambda/release/bootstrap
	zip -j target/lambda-rust.zip ./target/lambda/release/bootstrap

build-lambda-release: target/lambda-rust.zip

.PHONY: all clean build-lambda-release build-docker
