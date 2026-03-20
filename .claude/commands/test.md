# Run Tests

Run the tests for the specified language: $ARGUMENTS

Accepted values: `go`, `java`, `rust`, or `all` (runs all three).

## Steps for each language

### Go

1. Check if `go` is available:
   ```bash
   which go || go version
   ```
2. If missing, install via Homebrew:
   ```bash
   brew install go
   ```
3. Run the tests:
   ```bash
   cd go-kata && go test ./pricing/... -v
   ```

### Java

1. Check if a JDK is available and its version:
   ```bash
   java -version
   ```
2. If missing or below Java 17, install via Homebrew:
   ```bash
   brew install --cask temurin@17
   ```
   Then set JAVA_HOME:
   ```bash
   export JAVA_HOME=$(/usr/libexec/java_home -v 17)
   ```
3. Run the tests (Gradle wrapper handles its own download):
   ```bash
   cd java-kata && ./gradlew test
   ```
4. After the build, print a human-readable summary of passed/failed tests by reading the XML in `build/test-results/test/`.

### Rust

1. Check if `cargo` is available:
   ```bash
   which cargo || ~/.cargo/bin/cargo --version
   ```
2. If missing, install Rust via rustup (non-interactive):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
   source "$HOME/.cargo/env"
   ```
3. Run the tests:
   ```bash
   cd rust-kata && cargo test
   ```

## Reporting

After running tests for each requested language, print a concise summary:
- Language name
- Number of tests passed / total
- Any failures with their names and actual vs expected values

If a language's toolchain could not be installed, report it clearly but continue with other languages.
