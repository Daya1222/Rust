use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Status {
    Completed,
    Incomplete,
}

#[derive(Serialize, Deserialize, Clone)]
struct Task {
    number: u32,
    task: String,
    status: Status,
}

impl Task {
    fn new(number: u32, task: String, status: Option<Status>) -> io::Result<Self> {
        Ok(Task {
            number,
            task,
            status: status.unwrap_or(Status::Incomplete),
        })
    }

    /// Reads the JSON file and returns a vector of tasks
    fn read_file() -> io::Result<Vec<Self>> {
        if !std::path::Path::new("tasks.json").exists() {
            return Ok(Vec::new());
        }

        let mut file = File::open("tasks.json")?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        let task_list: Vec<Self> = serde_json::from_str(&content)?;
        Ok(task_list)
    }

    /// Saves a task to the JSON file, preventing duplicates by task text
    fn save(&self) -> io::Result<()> {
        let mut task_list = Task::read_file().unwrap_or_else(|_| Vec::new());
        if task_list.iter().any(|t| t.task == self.task) {
            return Ok(());
        }
        task_list.push(self.clone());
        let mut file = File::create("tasks.json")?;
        let content = serde_json::to_string_pretty(&task_list)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

/// Displays the menu and returns the user's choice
fn ask_user() -> io::Result<u8> {
    println!("-------------------------------");
    println!("1. Add Task");
    println!("2. Remove Task");
    println!("3. Edit Task");
    println!("4. View Tasks");
    println!("0. Exit");

    print!("Choose an option: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: u8 = input
        .trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(selection)
}

/// Adds a new task
fn add_task() -> io::Result<()> {
    let task_list = Task::read_file()?;
    let number: u32 = task_list.len() as u32 + 1;

    let mut task = String::new();
    print!("Enter the task detail: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut task)?;
    let cleaned_task: String = task.trim().to_string();

    let task_obj = Task::new(number, cleaned_task, None)?;
    task_obj.save()?;

    println!("Task added!");
    Ok(())
}

/// Edits a task's text or status
fn edit_task() -> io::Result<()> {
    let mut task_list: Vec<Task> = Task::read_file()?;
    view_task()?; // show current tasks

    let mut selection = String::new();
    print!("Select the task number to edit: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut selection)?;
    let index = selection
        .trim()
        .parse::<usize>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if index == 0 || index > task_list.len() {
        println!("No task at that number");
        return Ok(());
    }

    let task = &mut task_list[index - 1];

    println!("Selected task: {} | {:?}", task.task, task.status);
    println!("What do you want to edit?");
    println!("1. Task text");
    println!("2. Status");

    let mut choice = String::new();
    print!("Enter choice: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut choice)?;
    let choice = choice
        .trim()
        .parse::<u8>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    match choice {
        1 => {
            let mut new_task = String::new();
            print!("Enter new task description: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut new_task)?;
            task.task = new_task.trim().to_string();
            println!("Task text updated!");
        }
        2 => {
            println!("Select new status:");
            println!("1. Incomplete");
            println!("2. Completed");

            let mut status_choice = String::new();
            io::stdout().flush()?;
            io::stdin().read_line(&mut status_choice)?;
            let status_choice = status_choice
                .trim()
                .parse::<u8>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            task.status = match status_choice {
                1 => Status::Incomplete,
                2 => Status::Completed,
                _ => {
                    println!("Invalid status, keeping previous value");
                    task.status.clone()
                }
            };
            println!("Task status updated!");
        }
        _ => println!("Invalid choice"),
    }

    // Save updated tasks back to file
    let mut file = File::create("tasks.json")?;
    let content = serde_json::to_string_pretty(&task_list)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

/// Removes a task
fn remove_task() -> io::Result<()> {
    let mut task_list = Task::read_file()?;
    view_task()?; // show tasks

    let mut selection = String::new();
    print!("Select the task number to remove: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut selection)?;
    let index = selection
        .trim()
        .parse::<usize>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if index == 0 || index > task_list.len() {
        println!("No task at that number");
        return Ok(());
    }

    let removed = task_list.remove(index - 1);
    println!("Removed task: {}", removed.task);

    // Save updated tasks back to file
    let mut file = File::create("tasks.json")?;
    let content = serde_json::to_string_pretty(&task_list)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

/// Displays all tasks
fn view_task() -> io::Result<()> {
    let task_list = Task::read_file()?;

    println!("\n------------- TASKS ---------------");
    for (i, task) in task_list.iter().enumerate() {
        let status_icon = match task.status {
            Status::Completed => "[✔ ]",
            Status::Incomplete => "[  ]",
        };
        println!("{}. {} {}", i + 1, status_icon, task.task,);
    }
    println!("-----------------------------------\n");

    Ok(())
}

/// Main loop
fn main() {
    loop {
        let option = match ask_user() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        };

        match option {
            1 => {
                if let Err(e) = add_task() {
                    eprintln!("Failed to add task: {}", e);
                }
            }
            2 => {
                if let Err(e) = remove_task() {
                    eprintln!("Failed to remove task: {}", e);
                }
            }
            3 => {
                if let Err(e) = edit_task() {
                    eprintln!("Failed to edit task: {}", e);
                }
            }
            4 => {
                if let Err(e) = view_task() {
                    eprintln!("Failed to view tasks: {}", e);
                }
            }
            0 => break,
            _ => println!("Invalid Option"),
        }
    }
}
