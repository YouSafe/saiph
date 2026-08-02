EXE	:= saiph

.PHONY: all clean

all: 
	cargo rustc --release --package engine --bin engine -- --emit link="$(EXE)"

clean:
	cargo clean
	rm -f "$(EXE)"