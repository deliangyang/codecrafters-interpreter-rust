
```bash
cargo build --release
sudo perf record --call-graph=dwarf ./target/release/codecrafters-interpreter compile tests/opcode/fabonacci.lox
sudo perf report --stdio
```