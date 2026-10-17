use std::{io};

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
        self.website == website
    }
}

fn main() {
    let mut entries: Vec<PasswordEntry> = Vec::new();

    loop {
        println!("\n===== RustVault =====");
        println!("1. Add an entry");
        println!("2. View all entries");
        println!("3. Search for an entry");
        println!("4. Exit");

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

                println!("Enter the webiste: ");
                io::stdin()
                    .read_line(&mut website)
                    .expect("Failed to read webiste.");
                println!();

                println!("Enter the username: ");
                io::stdin()
                    .read_line(&mut username)
                    .expect("Failed to read username.");
                println!();

                println!("Enter the password: ");
                io::stdin()
                    .read_line(&mut password)
                    .expect("Failed to read password.");
                println!();

                let entry = PasswordEntry::new(
                    website.trim().to_string(),
                    username.trim().to_string(),
                    password.trim().to_string(),
                );

                entries.push(entry);

                println!("Entry added successfully!")
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
                        None => println!("\nNo entry found"),
                    }
                } else {
                    println!("No entries found!");
                }
            }
            "4" => {
                println!("\nExiting Rust Vault...");
                break;
            }
            _ => println!("\nInvalid choice. Try again!"),
        }
    }
}
