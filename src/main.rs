use sqlite::{Connection, State};
use std::{env, io, path::Path, process};

fn init() -> Connection {
    let conn = match Connection::open("config.db") {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    conn.execute("CREATE TABLE IF NOT EXISTS config (name TEXT, server TEXT)")
        .unwrap();
    conn
}

fn config(conn: &Connection) {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please provide an option");
        return;
    }

    match args[1].as_str() {
        "-f" => {
            if args.len() < 4 {
                println!("Please provide a directory and a name");
            } else if Path::new(&args[2]).is_file() {
                let file_path = &args[2];
                let name = &args[3];
                let mut stmt = conn
                    .prepare("INSERT INTO config (name, server) VALUES (?, ?)")
                    .unwrap();
                stmt.bind((1, name.as_str())).unwrap();
                stmt.bind((2, file_path.as_str())).unwrap();
                stmt.next().unwrap();
                println!("Configuration saved");
            } else {
                println!("{} is not a file", args[2]);
            }
        }
        "-u" => {
            if args.len() < 4 {
                println!("Please provide a name to update and a new server path");
            } else {
                let update_name = &args[2];
                let new_server = &args[3];
                let mut stmt = conn
                    .prepare("UPDATE config SET server = ? WHERE name = ?")
                    .unwrap();
                stmt.bind((1, new_server.as_str())).unwrap();
                stmt.bind((2, update_name.as_str())).unwrap();

                match stmt.next() {
                    Ok(_) => println!("Configuration updated for name: {}", update_name),
                    Err(_) => println!(
                        "Failed to update configuration or no configuration found with name: {}",
                        update_name
                    ),
                }
            }
        }
        "-s" => {
            if args.len() < 3 {
                println!("Please provide a name to search");
            } else {
                let search_name = &args[2];
                let mut stmt = conn
                    .prepare("SELECT name, server FROM config WHERE name = ?")
                    .unwrap();
                stmt.bind((1, search_name.as_str())).unwrap();

                if let State::Row = stmt.next().unwrap() {
                    let name: String = stmt.read(0).unwrap();
                    let server: String = stmt.read(1).unwrap();
                    println!("Starting Server {}", name);
                    let cwd = env::current_dir().unwrap();
                    let path = Path::new(&cwd).join(server);
                    start(path.to_str().unwrap());
                } else {
                    println!("No configuration found with name: {}", search_name);
                }
            }
        }
        "-d" => {
            if args.len() < 3 {
                println!("Please provide a name to delete");
            } else {
                let delete_name = &args[2];
                let mut stmt = conn.prepare("DELETE FROM config WHERE name = ?").unwrap();
                stmt.bind((1, delete_name.as_str())).unwrap();

                match stmt.next() {
                    Ok(_) => println!("Configuration deleted for name: {}", delete_name),
                    Err(_) => println!(
                        "Failed to delete configuration or no configuration found with name: {}",
                        delete_name
                    ),
                }
            }
        }
        "-h" => {
            println!("Usage:");
            println!("  server-mgmt -f <file> <name>");
            println!("  server-mgmt -s <name>");
            println!("  server-mgmt -h");
        }
        _ => println!("Invalid option"),
    }
}

fn start(server: &str) {
    match process::Command::new(&server)
        .arg(&server)
        .stdout(io::stdout())
        .stderr(io::stderr())
        .spawn()
    {
        Ok(mut child) => {
            child.wait().unwrap();
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn main() {
    let conn = init();
    config(&conn);
}
