use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize)]
struct PasswordEntry {
    website: String,
    username: String,
    password: String,
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

fn save_entries(entries: &[PasswordEntry]) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(entries)?;

    std::fs::write("vault.json", json)?;

    Ok(())
}

fn load_entries() -> Result<Vec<PasswordEntry>, Box<dyn std::error::Error>> {
    let path = std::path::Path::new("vault.json");

    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(path)?;

    let entries: Vec<PasswordEntry> = serde_json::from_str(&contents)?;

    Ok(entries)
}

fn main() {
    let mut entries = match load_entries() {
        Ok(entries) => {
            println!("Loaded {} saved entries.", entries.len());
            entries
        }
        Err(error) => {
            eprintln!("Failed to load vault: {error}");
            return;
        }
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
                    println!("Enter the password: ");

                    password.clear();

                    io::stdin()
                        .read_line(&mut password)
                        .expect("Failed to read password.");

                    if password.trim().is_empty() {
                        println!("Password cannot be empty! Try again.")
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
                match save_entries(&entries) {
                    Ok(()) => println!("Vault saved successfully!"),
                    Err(error) => eprintln!("Failed to save entries: {error}"),
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

                    save_entries(&entries);
                    match save_entries(&entries) {
                        Ok(()) => println!("Vault updated successfully!"),
                        Err(error) => eprintln!("Failed to save vault: {error}"),
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
