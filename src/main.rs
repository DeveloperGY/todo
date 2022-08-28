/**
 *  Todo List
 * 
 *  Todo List is a cli tracker for things you need to do, whether it be for programming,
 *  daily scheduling, or general note taking.
 * 
 * 
 *  Developed by FlyingParrot225
 */

// todo global add This is it

// todo global list
/*
    (1) - This is it
*/

// todo --global remove 1

// todo --global list
/*
    None
*/

extern crate dirs;

use std::{env, fs::{File, remove_file}, path::Path, io::prelude::*/*, thread::LocalKey*/};

/// Locations contains the locations of different lists [Global List, or Local List]
enum Locations
{
    /// Global refers to the global Todo of the user, located in their home folder
    Global,
    
    /// Local refers to the local Todo of the current directory
    Local,
}

/// Command contains all the possible commands you can do with Todo
enum Command
{
    /// Adds an item to the Todo
    Add,

    /// Removes an item from the Todo
    Remove,

    /// Lists the items in the Todo
    List,

    /// Creates a Todo
    Create,

    /// Deletes a Todo
    Delete
}

/// The entry point into the Todo program
fn main()
{
    // Retrieve Arguments
    let args: Vec<String> = env::args().collect();

    // Panic if there is an invalid number of arguments
    if args.len() < 3 || args.len() > 4
    {
        panic!("Error: Invalid number of arguments!");
    }

    // Retrieve the file location
    let location = match get_location(args[1].as_str())
    {
        Some(loc)    => loc,
        None                    => panic!("Error: Invalid Location!")
    };

    // Retrieve the command
    let command = match get_command(args[2].as_str())
    {
        Some(command) => command,
        None          => panic!("Error: Invalid Command!")
    };

    // Set the global path
    let global = match dirs::home_dir()
    {
        Some(home) => home.join(Path::new(".todo")),
        None       => panic!("Error: Could not find home directory!"),
    };

    // Set the local path
    let local = Path::new(".todo").to_path_buf();

    // change path buffers to paths with shadowing
    let global = global.as_path();
    let local = local.as_path();

    // Retrieve true path
    let path = match location
    {
        Locations::Global => global,
        Locations::Local => local,
    };

    // Execute command
    match command
    {
        Command::Add        => {add(&path, &args)},
        Command::Remove     => {remove(&path, &args)},
        Command::List       => {list(&path)},
        Command::Create     => {create(&path)},
        Command::Delete     => {delete(&path)},
    }
}

/// Retrieves the Todo location from the arguments
fn get_location(arg: &str) -> Option<Locations>
{
    match arg.to_lowercase().as_str()
    {
        "global"    => Some(Locations::Global),
        "local"     => Some(Locations::Local),
        &_          => None
    }
}

/// Retrieves the Todo command from the arguments
fn get_command(arg: &str) -> Option<Command>
{
    match arg.to_lowercase().as_str()
    {
        "add"       => Some(Command::Add),
        "remove"    => Some(Command::Remove),
        "list"      => Some(Command::List),
        "create"    => Some(Command::Create),
        "delete"    => Some(Command::Delete),
        &_          => None
    }
}

/// Adds an item to the specified Todo
fn add(path: &Path, args: &Vec<String>)
{
    if args.len() < 4
    {
        panic!("Error: Please pass your item as an argument!");
    }

    let item = args[3].as_bytes();
    let mut newline: [u8; 1] = [0];
    '\n'.encode_utf8(&mut newline);

    let mut file = std::fs::OpenOptions::new().append(true).open(path).expect("Error: Failed to open Todo! (Did you create one?)");
    file.write_all(item).expect("Error: Failed to add item to Todo!");
    file.write_all(&newline).expect("Error: Failed to add item to Todo!");
}

fn remove(path: &Path, args: &Vec<String>)
{
    if args.len() < 4
    {
        panic!("Error: Please pass your item index as an argument!");
    }

    let mut file: File = File::open(path).expect("Error: Failed to open Todo! (Did you create one?)");
    
    let mut contents: String = String::new();
    file.read_to_string(&mut contents).expect("Error: Failed to read Todo!");

    let mut lines = contents.lines();
    let mut line = lines.next();

    let mut line_vector: Vec<String> = Vec::new();

    let index: usize = match args[3].parse()
    {
        Ok(result) => result,
        Err(_) => panic!("Error: Please pass an unsigned integer as the item index!")
    };

    let mut current_index: usize = 0;

    while line != None
    {
        if current_index != index - 1
        {
            line_vector.push(line.unwrap().to_string());
        }
        line = lines.next();
        current_index += 1;
    }

    let mut newline: [u8; 1] = [0];
    '\n'.encode_utf8(&mut newline);

    let mut file: File = File::create(path).expect("Error: Failed to remove item from Todo!");
    for item in line_vector
    {
        file.write_all(item.as_bytes()).expect("Error: Failed to reconstruct Todo!");
        file.write_all(&newline).expect("Error: Failed to add item to Todo!");
    }
}

/// Lists the items in the specified Todo
fn list(path: &Path)
{
    let mut file: File = File::open(path).expect("Error: Failed to open Todo! (Did you create one?)");
    
    let mut contents: String = String::new();
    file.read_to_string(&mut contents).expect("Error: Failed to read Todo!");

    let mut lines = contents.lines();
    let mut line = lines.next();
    let mut num: u64 = 1;

    while line != None
    {
        println!("({}) - {}", num, line.unwrap().to_string());
        num += 1;
        line = lines.next();
    }
}

/// Creates a new Todo
fn create(path: &Path)
{
    let _file: File = match File::open(path)
    {
        Ok(_file) => panic!("Error: Failed to create Todo, Todo already exists!"),
        Err(_)    => File::create(path).expect("Error: Failed to create Todo!")
    };
}

/// Deletes a Todo
fn delete(path: &Path)
{
    remove_file(path).expect("Error: Failed to delete Todo! (Are you sure it existed?)");
}