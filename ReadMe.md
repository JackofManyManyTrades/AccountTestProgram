AccountTestProgram
A high-performance Rust utility designed to ingest, process, and summarize account data from CSV files. This project demonstrates custom binary configuration and efficient data handling using the Rust ecosystem.
📖 Table of Contents
Installation
Features
Project Structure
Configuration
Usage
Dependencies

Installation: 
Clone the repository:
PowerShell: 
	git clone <your-repository-url>
	cd AccountTestProgram
Build the project:
To build the executable in debug mode:
PowerShell
	cargo build
To build a performance-optimized version:
PowerShell
	cargo build --release

Features:
Custom Binary Naming: Configured to output a specific executable name (account_test_program) regardless of the package name.
CSV Processing: Leverages the csv crate for fast, memory-efficient data parsing.
File System Traversal: Uses walkdir to handle complex file structures and automated data discovery.
Safety First: Built with Rust's strict type system to ensure data integrity during ingestion.

📂 Project Structure
The project is structured to separate configuration from implementation:
Plaintext
AccountTestProgram/
├── src/
│   └── account_test_program.rs  # Main application logic
├── target/                      # Compiled artifacts (auto-generated)
├── Cargo.toml                   # Project manifest & metadata
├── Cargo.lock                   # Deterministic dependency locking
└── funidea_clean.csv            # Sample data for processing

Configuration: 
This project uses a custom [[bin]] section in the Cargo.toml to define the executable path. This allows the source file to live in src/account_test_program.rs while maintaining a clean output name.
Ini, TOML
[[bin]]
name = "account_test_program"
path = "src/account_test_program.rs"
🛠 Usage
To run the program directly via Cargo:
PowerShell
cargo run
Alternatively, you can run the compiled binary directly from the target folder:
PowerShell
./target/debug/account_test_program.exe
Note: Ensure your input CSV files (like funidea_clean.csv) are located in the path specified within the source code (currently pointing to E:\ based on recent builds).

Dependencies: 
Crate
Version
Purpose
csv
1.1.0
Provides the underlying engine for reading/writing CSV records.
walkdir
2.5.0