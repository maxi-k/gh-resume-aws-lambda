all: clean build-lambda-release

clean:
	rm -rf target

target/x86_64-unknown-linux-musl/release/bootstrap:
	cargo build --release --target x86_64-unknown-linux-musl



target/rust.zip: target/x86_64-unknown-linux-musl/release/bootstrap
	zip -j target/rust.zip ./target/x86_64-unknown-linux-musl/release/bootstrap

build-lambda-release: target/rust.zip


.PHONY: all clean build-lambda-release
