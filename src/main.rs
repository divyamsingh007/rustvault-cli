use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::Argon2;
use base64::{Engine, engine::general_purpose::STANDARD};
use getrandom::fill;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize)]
struct PasswordEntry {
    website: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct EncryptedVault {
    version: u32,
    salt: String,
    nonce: String,
    ciphertext: String,
}

impl PasswordEntry {
    fn new(website: String, username: String, password: String) -> Self {
        Self {
            website,
            username,
            password,
        }
    }

    fn display(&self) {
        println!("Website: {}", self.website);
        println!("Username: {}", self.username);
        println!("Password: [hidden]");
        println!();
    }

    fn belongs_to(&self, website: &str) -> bool {
        self.website.eq_ignore_ascii_case(website.trim())
    }

    fn reveal(&self) {
        println!("Website: {}", self.website);
        println!("Username: {}", self.username);
        println!("Password: {}", self.password);
        println!();
    }
}

fn get_master_password() -> Result<String, Box<dyn std::error::Error>> {
    let master_password = rpassword::prompt_password("Enter Master Password: ")?;
    Ok(master_password)
}

fn generate_salt() -> Result<[u8; 16], getrandom::Error> {
    let mut salt = [0u8; 16];

    fill(&mut salt)?;

    Ok(salt)
}

fn derive_key(master_password: &str, salt: &[u8; 16]) -> Result<[u8; 32], argon2::Error> {
    let argon2 = Argon2::default();

    let mut key = [0u8; 32];

    argon2.hash_password_into(master_password.as_bytes(), salt, &mut key)?;

    Ok(key)
}

fn encrypt_data(
    plaintext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;

    let nonce = Nonce::try_from(nonce.as_slice())?;

    let ciphertext = cipher.encrypt(&nonce, plaintext)?;

    Ok(ciphertext)
}

fn encrypt_entries(
    entries: &[PasswordEntry],
    key: &[u8; 32],
) -> Result<(Vec<u8>, [u8; 12]), Box<dyn std::error::Error>> {
    let plaintext = serde_json::to_vec(entries)?;

    let mut nonce = [0u8; 12];
    fill(&mut nonce)?;

    let ciphertext = encrypt_data(&plaintext, key, &nonce)?;

    Ok((ciphertext, nonce))
}

fn save_encrypted_vault(
    entries: &[PasswordEntry],
    key: &[u8; 32],
    salt: &[u8; 16],
) -> Result<(), Box<dyn std::error::Error>> {
    let (ciphertext, nonce) = encrypt_entries(entries, key)?;

    let encrypted_vault = EncryptedVault {
        version: 1,
        salt: STANDARD.encode(salt),
        nonce: STANDARD.encode(nonce),
        ciphertext: STANDARD.encode(ciphertext),
    };

    let json = serde_json::to_string_pretty(&encrypted_vault)?;

    std::fs::write("vault.enc.json", json)?;

    Ok(())
}

fn load_encrypted_vault(
    master_password: &str,
) -> Result<Vec<PasswordEntry>, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string("vault.enc.json")?;

    let vault: EncryptedVault = serde_json::from_str(&contents)?;

    if vault.version != 1 {
        return Err("Unsupported vault version".into());
    };

    let salt: [u8; 16] = STANDARD
        .decode(&vault.salt)?
        .try_into()
        .map_err(|_| "Invalid salt length")?;

    let nonce: [u8; 12] = STANDARD
        .decode(&vault.nonce)?
        .try_into()
        .map_err(|_| "Invalid nonce length")?;

    let ciphertext = STANDARD.decode(&vault.ciphertext)?;

    let key = derive_key(master_password, &salt)?;

    let plaintext = decrypt_data(&ciphertext, &key, &nonce)?;

    let entries: Vec<PasswordEntry> = serde_json::from_slice(&plaintext)?;

    Ok(entries)
}

fn vault_exists() -> bool {
    std::path::Path::new("vault.enc.json").exists()
}

fn unlock_vault() -> Result<(Vec<PasswordEntry>, [u8; 32], [u8; 16]), Box<dyn std::error::Error>> {
    let master_password = get_master_password()?;

    let entries = load_encrypted_vault(&master_password)?;

    let contents = std::fs::read_to_string("vault.enc.json")?;
    let vault: EncryptedVault = serde_json::from_str(&contents)?;

    let salt: [u8; 16] = STANDARD
        .decode(&vault.salt)?
        .try_into()
        .map_err(|_| "Invalid salt length")?;

    let key = derive_key(&master_password, &salt)?;

    println!("Vault unlocked successfully!");

    Ok((entries, key, salt))
}

fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;

    let nonce = Nonce::try_from(nonce.as_slice())?;
    let plaintext = cipher.decrypt(&nonce, ciphertext)?;

    Ok(plaintext)
}

fn main() {
    let (mut entries, key, salt) = if vault_exists() {
        println!("Existing encrypted vault found!");

        match unlock_vault() {
            Ok((entries, key, salt)) => {
                println!("Successfully loaded {} entries.", entries.len());
                (entries, key, salt)
            }
            Err(error) => {
                eprintln!("Failed to unlock vault: {error}");
                return;
            }
        }
    } else {
        println!("No encrypted vault found. Creating a new vault.");

        let master_password = match get_master_password() {
            Ok(password) if !password.is_empty() => password,
            Ok(_) => {
                eprintln!("Master password cannot be empty!");
                return;
            }
            Err(error) => {
                eprintln!("Failed to read master password: {error}");
                return;
            }
        };

        let salt = match generate_salt() {
            Ok(salt) => salt,
            Err(error) => {
                eprintln!("Failed to generate salt: {error}");
                return;
            }
        };

        let key = match derive_key(&master_password, &salt) {
            Ok(key) => key,
            Err(error) => {
                eprintln!("Key derivation failed: {error}");
                return;
            }
        };

        let entries = Vec::new();

        if let Err(error) = save_encrypted_vault(&entries, &key, &salt) {
            eprintln!("Failed to create encrypted vault: {error}");
            return;
        }

        println!("New encrypted vault created successfully!");

        (entries, key, salt)
    };

    loop {
        println!("\n===== RustVault =====");
        println!("1. Add an entry");
        println!("2. View all entries");
        println!("3. Search for an entry");
        println!("4. Delete an entry");
        println!("5. Reveal password");
        println!("6. Exit RustVault");

        let mut choice = String::new();

        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read input.");

        match choice.trim() {
            "1" => {
                println!("\nAdding new entry...");

                let mut website = String::new();
                let mut username = String::new();
                let mut password = String::new();

                loop {
                    println!("Enter the website:");

                    website.clear();

                    io::stdin()
                        .read_line(&mut website)
                        .expect("Failed to read website.");

                    if website.trim().is_empty() {
                        println!("Website cannot be empty! Try again.");
                    } else {
                        println!("Website accepted!");
                        break;
                    }
                }
                println!();

                loop {
                    println!("Enter the username: ");

                    username.clear();
                    io::stdin()
                        .read_line(&mut username)
                        .expect("Failed to read username.");

                    if username.trim().is_empty() {
                        println!("Username cannot be empty! Try again.");
                    } else {
                        println!("Username accepted!");
                        break;
                    }
                }
                println!();

                loop {
                    password.clear();
                    match rpassword::prompt_password("Enter the password: ") {
                        Ok(input) => password = input,
                        Err(error) => {
                            eprintln!("Failed to read password: {error}");
                            return;
                        }
                    }
                    if password.trim().is_empty() {
                        println!("Password cannot be empty! Try again.");
                    } else {
                        println!("Password accepted!");
                        break;
                    }
                }
                println!();

                let entry = PasswordEntry::new(
                    website.trim().to_lowercase(),
                    username.trim().to_string(),
                    password.trim().to_string(),
                );

                if let Some(existing) = entries
                    .iter_mut()
                    .find(|existing| existing.belongs_to(entry.website.as_str()))
                {
                    existing.username = entry.username;
                    existing.password = entry.password;
                    println!("Credentials updated successfully!")
                } else {
                    entries.push(entry);
                }
                match save_encrypted_vault(&entries, &key, &salt) {
                    Ok(()) => println!("Encrypted vault saved successfully!"),
                    Err(error) => eprintln!("Failed to save encrypted vault: {error}"),
                }
            }
            "2" => {
                if !entries.is_empty() {
                    println!("\nShowing all entries...\n");

                    for entry in &entries {
                        entry.display();
                        println!();
                    }

                    println!("All entries showed!");
                } else {
                    println!("\nNo entries found!");
                }
            }
            "3" => {
                if !entries.is_empty() {
                    println!("\nSearch an entry");

                    let mut user_website_input = String::new();

                    println!("Enter the webiste to search creadentials: ");
                    io::stdin()
                        .read_line(&mut user_website_input)
                        .expect("Failed to read input!");

                    let result = entries
                        .iter()
                        .find(|entry| entry.belongs_to(&user_website_input.trim()));

                    match result {
                        Some(entry) => {
                            println!("\nEntry found.");
                            entry.display();
                        }
                        None => println!("\nNo entry found!"),
                    }
                } else {
                    println!("No entries found!");
                }
            }
            "4" => {
                if entries.is_empty() {
                    println!("\nNo entries found for deletion!");
                } else {
                    let mut user_delete_input = String::new();
                    println!("\nEnter the website whose credentials you want to delete:");
                    io::stdin()
                        .read_line(&mut user_delete_input)
                        .expect("Failed to read input.");

                    let index = entries
                        .iter()
                        .position(|entry| entry.belongs_to(user_delete_input.trim()));

                    match index {
                        Some(index) => {
                            entries.remove(index);
                            println!("\nEntry removed successfully!");
                        }
                        None => println!("Failed to remove the entry!"),
                    }

                    match save_encrypted_vault(&entries, &key, &salt) {
                        Ok(()) => println!("Encrypted vault updated successfully!"),
                        Err(error) => eprintln!("Failed to update encrypted vault: {error}"),
                    }
                }
            }
            "5" => {
                if entries.is_empty() {
                    println!("\nNo entries found for deletion!");
                } else {
                    let mut user_password_input = String::new();
                    println!("\nEnter the website to reveal its password: ");
                    io::stdin()
                        .read_line(&mut user_password_input)
                        .expect("Failed to read input");

                    let result = entries
                        .iter()
                        .find(|entry| entry.belongs_to(user_password_input.trim()));

                    match result {
                        Some(entry) => {
                            println!();
                            entry.reveal();
                        }
                        None => println!("No credentials found for this website."),
                    }
                }
            }
            "6" => {
                println!("\nExiting Rust Vault...");
                break;
            }
            _ => println!("\nInvalid choice. Try again!"),
        }
    }
}
