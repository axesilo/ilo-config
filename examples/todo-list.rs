//! To-do list CLI that uses ilo-config to store its data on disk.
//!
//! Usage:
//!
//! ```sh
//! cargo run --example todo-list -- add "Finish sketch of skyeels"
//! cargo run --example todo-list -- add "Get directions to the Palanaeum"
//! cargo run --example todo-list -- list
//! cargo run --example todo-list -- do 1  # Mark "Finish sketch of skyeels" complete
//! ```
//!
//! # Developer Notes
//!
//! Typically, ilo-config is used to store links and credentials to other data stores, but for small
//! programs it's an easy way to load and save data on disk directly.  It's up to the client to
//! implement delete protection and backups, however.
//!
//! This example does not use any external crates other than ilo-config.  In particular, the config
//! data type is typed as a Vec<(String, bool)> in order to not need Serde.  However, it is
//! recommended to use strong, domain-specific typing on the config data type if possible (see
//! the Quickstart for an example).
use std::env;

use ilo_config::Config;

fn main() {
    let mut config: Config<Vec<(String, bool)>> =
        Config::load("axesilo-example-todo-list").expect("Failed to load todo list!");
    let todo_list = config.data_mut();

    let mut args = env::args().skip(1);
    let command = args.next();
    let item = args.next();

    let mut is_data_dirty = false;

    match (command.as_deref(), item.as_deref()) {
        (Some("add"), Some(item)) => {
            todo_list.push((item.into(), false));
            println!(
                "Added \"{}\" to the todo list at position {}.",
                item,
                todo_list.len()
            );
            is_data_dirty = true;
        }
        (Some("add"), None) => println!(
            "Please enter the description of the item, e.g. `do \"Finish sketch of skyeels\"`"
        ),
        (Some("list"), _) => {
            for (i, item) in todo_list.iter().enumerate() {
                println!(
                    "{:8} | {:>6} | {}",
                    if item.1 { "(DONE)" } else { "" },
                    i + 1,
                    item.0
                );
            }
        }
        (Some("do"), Some(key)) => {
            let todo_number: i32 = key
                .parse()
                .expect("Please enter the integer index of the todo item to complete.");
            let index = (todo_number - 1) as usize;
            if todo_number < 1 || todo_number > todo_list.len() as i32 || todo_list[index].1 {
                println!("Please enter the index of an incomplete item.");
            } else {
                todo_list[index].1 = true;
                println!("Marked \"{}\" as complete!", todo_list[index].0);
                is_data_dirty = true;
            }
        }
        (Some("do"), None) => {
            println!("Please enter the index of the item to mark as done, e.g. `1`.")
        }
        _ => println!("Please enter a valid command (`list`, `add`, or `do`)."),
    }

    if is_data_dirty {
        config.save().expect("Failed to save todo list!");
    }
}
