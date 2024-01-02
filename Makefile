all: clean build-lambda-release

BINARY := target/lambda/bootstrap/bootstrap
ZIP := target/lambda-rust.zip

clean:
	rm -f $BINARY
	rm -f $ZIP

$(BINARY):
	cargo lambda build --release --arm64

$(ZIP): $(BINARY)
	zip -j $(ZIP) $(BINARY)

build-lambda-release: $(ZIP)

.PHONY: all clean build-lambda-release 
