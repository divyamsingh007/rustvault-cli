# RustVault

A lightweight command-line password manager built with Rust. Store, search, update, and manage credentials in an encrypted local vault.

### Features

* AES-256-GCM encryption
* Argon2 key derivation
* Master password protection
* Credential management
* Encrypted local storage

### Getting Started

Install [Rust](https://www.rust-lang.org/tools/install), clone the repository, and navigate to the project directory:

```bash
git clone <your-repository-url>
cd RustVault
cargo run
```

Create a master password on the first launch. Use it to unlock your vault in future sessions.

### Usage

Run the application:

```bash
cargo run
```

Use the interactive menu to manage your credentials:

| Operation | Example                                    |
| --------- | ------------------------------------------ |
| Add       | Website: `github.com`, Username: `user123` |
| Search    | Search for `github.com`                    |
| Update    | Change an existing username or password    |
| View      | Browse saved credentials                   |
| Reveal    | Display a stored password                  |
| Delete    | Remove an entry                            |

Your encrypted vault is stored locally in `vault.enc.json`.

### Tech Stack

Rust, AES-256-GCM, Argon2, Serde, JSON.

### Note

RustVault is a learning project and has not undergone an independent security audit. Keep your master password safe; forgotten passwords cannot recover your vault.
