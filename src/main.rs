use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::process::exit;

const TASKS_DIR: &str = "./tasks";

fn tasks_buffer(operation: &str, task: Option<&Value>, buffer: &mut Vec<Value>) {
    match operation {
        "add" => {
            if let Some(task) = task {
                buffer.push(task.clone());
            } else {
                println!("No task provided to add.");
            }
        }
        "del" => {
            if let Some(task) = task {
                if let Some(pos) = buffer.iter().position(|t| t == task) {
                    buffer.remove(pos);
                } else {
                    println!("Task not found in buffer.");
                }
            } else {
                println!("No task provided to delete.");
            }
        }
        "clear" => {
            buffer.clear();
            println!("Buffer cleared.");
        }
        _ => {
            println!("Unknown operation. Available operations: add, del, clear.");
        }
    }
}

fn write_tasks_to_file(tasks: &[Value]) {
    fs::create_dir_all(TASKS_DIR).expect("Failed to create tasks directory");

    for task in tasks {
        let task_id = match task.get("id") {
            Some(id) => {
                if let Some(id_num) = id.as_u64() {
                    id_num.to_string()
                } else {
                    "unknown_id".to_string()
                }
            }
            None => "unknown_id".to_string(),
        };
        

        let file_path = format!("{}/task_{}.json", TASKS_DIR, task_id);

        let task_json = serde_json::to_string_pretty(task)
            .expect("Failed to serialize task to JSON");

        let mut file = File::create(&file_path).expect("Error creating file");
        file.write_all(task_json.as_bytes())
            .expect("Failed to write to file");
    }
}

fn read_tasks_from_file() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut tasks = Vec::new();

    if let Ok(entries) = fs::read_dir(TASKS_DIR) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                if path.is_file() {
                    let file = File::open(&path).expect("Failed to open task file");
                    
                    let task: Value = serde_json::from_reader(file)?;
                    tasks.push(task);
                }
            }
        }
    }

    Ok(tasks)
}

fn command_processor(command: &str, buffer: &mut Vec<Value>) {
    match command {
        "exit" => {
            println!("Exiting the program...");
            exit(0);
        }
        "help" => {
            println!("Available commands:");
            println!("- exit: Exit the program");
            println!("- help: Show this help message");
            println!("- new: Create a new task");
            println!("- save: Save tasks to file");
            println!("- clear: Clear the task buffer");
        }
        "new" => {
            let mut task_name = String::new();
            let mut task_context = String::new();
            let mut task_level = String::new();
            
            print!("Write task name: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut task_name)
                .expect("Failed to read task name");
            task_name = task_name.trim().to_string();

            print!("Write task context: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut task_context)
                .expect("Failed to read task context");
            task_context = task_context.trim().to_string();

            print!("Write task level (integer 1-5): ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut task_level)
                .expect("Failed to read task level");
            let task_level: u8 = match task_level.trim().parse() {
                Ok(level) if level >= 1 && level <= 5 => level,
                _ => {
                    println!("Invalid task level. Please enter an integer between 1 and 5.");
                    return;
                }
            };

            let task = serde_json::json!({
                "id": buffer.len() + 1,
                "name": task_name,
                "context": task_context,
                "level": task_level
            });

            tasks_buffer("add", Some(&task), buffer);
        }
        "save" => {
            write_tasks_to_file(buffer);
            println!("Tasks saved to files.");
        }
        "clear" => {
            tasks_buffer("clear", None, buffer);
            println!("Buffer is clear.")
        }
        "list" => {
            for task in buffer {
                let task_id = match task.get("id") {
                    Some(id) => {
                        if let Some(id_num) = id.as_u64() {
                            id_num.to_string()
                        } else {
                            "unknown_id".to_string()
                        }
                    }
                    None => "unknown_id".to_string(),
                };

                let task_name = match task.get("name") {
                    Some(name) => name.as_str().unwrap_or("Unnamed Task"),
                    None => "Unnamed Task",
                };

                let task_context = match task.get("context") {
                    Some(context) => context.as_str().unwrap_or("No context provided"),
                    None => "No context provided",
                };

                let task_level = match task.get("level") {
                    Some(level) => level.as_u64().unwrap_or(0),
                    None => 0,
                };

                println!("----- Task ID: {} -----", task_id);
                println!("Name: {}", task_name);
                println!("Context: {}", task_context);
                println!("Level: {}", task_level);
                println!("----------------------------");
            }
        }

        _ => {
            println!("Unknown command: '{}'. Type 'help' for a list of commands.", command);
        }
    }
}

fn main() {
    let mut buffer: Vec<Value> = Vec::new();

    match read_tasks_from_file() {
        Ok(tasks) => {
            for task in tasks {
                tasks_buffer("add", Some(&task), &mut buffer);
            }
        }
        Err(e) => {
            println!("Failed to read tasks from file: {}", e);
        }
    }
    
    loop {
        let mut command = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read command");
        let command = command.trim();
        command_processor(command, &mut buffer);
    }
}