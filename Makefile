.PHONY: all test check-example clean

all:
	cargo build --release --offline
	cp target/release/parser parser

test:
	cargo test --offline

check-example: all
	./parser fixtures/example.txt > target/example.actual.txt
	diff -u fixtures/example.expected.txt target/example.actual.txt

clean:
	cargo clean
	rm -f parser
