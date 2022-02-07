mod fmt;

use std::fs;
use clap::{ErrorKind, IntoApp, Parser};
use scylla::{Session, SessionBuilder, QueryResult};
use std::io::{Error, Write};
use std::time::Duration;
use std::borrow::Cow;
use std::fmt::Display;


#[derive(Parser, Debug)]
#[clap(about = "I am a cqlsh alternative, just pass `-h`")]
struct Cli {
    #[clap(default_value = "127.0.0.1:9042")]
    host: String,

    #[clap(long, short, help = "Execute the given statement, then exit")]
    execute: Option<String>,

    #[clap(long, short, help = "Execute commands from the given file, then exit")]
    file: Option<String>,

    #[clap(long, short, help = "Username to authenticate against Cassandra with")]
    user: Option<String>,

    #[clap(long, short, help = "Password to authenticate against Cassandra with, should be used in conjunction with --user")]
    password: Option<String>,

    #[clap(long, short, help = "Keyspace to authenticate to")]
    keyspace: Option<String>,

    #[clap(long, default_value = "2000", help = "Specify the connection timeout in seconds (defaults to 2s)")]
    connect_timeout: u64
}

fn print_row<T: Display>(data: &Vec<T>, width: &Vec<usize>) {
    let col_last_index = width.len() - 1;
    for (text, (width, col)) in data.iter().zip(width.iter().cycle().zip((0..width.len()).cycle())) {
        if col == col_last_index {
            println!("| {:<1$} |", text, width);
        } else {
            print!("| {:<1$} ", text, width);
        }
    }
}

fn display_query_result(query_result: &QueryResult) {
    let alt_row = Vec::new();
    let rows = query_result.rows.as_ref().unwrap_or(&alt_row);

    if rows.len() > 0 {
        let mut width: Vec<usize> = query_result.col_specs.iter()
            .map(|spec| spec.name.chars().count()).collect();
        let table_head: Vec<&String> = query_result.col_specs.iter()
            .map(|spec| { &spec.name }).collect();
        let mut table: Vec<Cow<str>> = Vec::new();

        // Generate Table
        for col_spec in rows {
            for (col, w) in col_spec.columns.iter().zip(width.iter_mut()) {
                let s = fmt::fmt_opt(col);
                let l = s.chars().count();
                if l > *w {
                    *w = l;
                }
                table.push(s);
            }
        }

        // Print Table
        println!();
        print_row(&table_head, &width);
        let col_last_index = width.len() - 1;
        for (width, col) in width.iter().zip((0..width.len()).cycle()) {
            if col == col_last_index {
                println!("+-{:-^1$}-+", "-", width);
            } else {
                print!("+-{:-^1$}-", "-", width);
            }
        }
        print_row(&table, &width);
        println!();
    } else {
        println!("\nEmpty Result Set\n");
    }
}

async fn execute_query(session: &Session, query: &str) {
    match session.query(query, &[]).await {
        Ok(query_result) => {
            display_query_result(&query_result)
        }
        Err(err) => {
            println!("\n{}\n", err);
        }
    }
}

async fn session_information(args: &Cli, session: &Session) {
    let result = session.query("SELECT cluster_name, cql_version, release_version FROM system.local", &[]).await
        .expect("could not connect to database");
    let rows = result.rows.as_ref().unwrap();
    let row = rows.get(0).unwrap();
    let cols = &row.columns;
    let cluster_name = cols.get(0).unwrap().as_ref().unwrap();
    let cql_version = cols.get(1).unwrap().as_ref().unwrap();
    let release_version = cols.get(2).unwrap().as_ref().unwrap();
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    println!("Connected to {} at {}.", fmt::fmt(cluster_name), &args.host);
    println!("[ cqlsh-rs {} | Cassandra {} | CQL spec {} | Native protocol v4 ]",
             VERSION, fmt::fmt(release_version), fmt::fmt(cql_version))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Cli = Cli::parse();
    if args.user.is_some() && args.password.is_none() || args.user.is_none() && args.password.is_some() {
        let mut app = Cli::into_app();
        app.error(
            ErrorKind::ArgumentConflict,
            "You need to specify username AND password",
        ).exit();
    }
    if args.execute.is_some() && args.file.is_some() {
        let mut app = Cli::into_app();
        app.error(
            ErrorKind::ArgumentConflict,
            "You can not provide a file and a command to execute, please provide -f OR -e",
        ).exit();
    }

    let mut session_builder = SessionBuilder::new();
    session_builder = session_builder.known_node(&args.host);
    if args.user.is_some() && args.password.is_some() {
        session_builder = session_builder.user(args.user.as_ref().unwrap(), args.password.as_ref().unwrap());
    }
    if args.keyspace.is_some() {
        session_builder = session_builder.use_keyspace(args.keyspace.as_ref().unwrap(), false);
    }
    session_builder = session_builder.connection_timeout(Duration::from_millis(args.connect_timeout));

    let session: Session = session_builder.build().await
        .expect("could not connect to database");
    session_information(&args, &session).await;

    if let Some(file_path) = &args.file {
        let file_content = fs::read_to_string(file_path)
            .expect("Could not read file");
        for execute in file_content.replace("\n", "").split(";") {
            if !execute.is_empty() && execute.len() > 1 {
                println!("cqlsh> {};", execute);
                execute_query(&session, execute).await;
            }
        }
    } else {
        if let Some(execute) = &args.execute {
            for execute in execute.split(";") {
                if !execute.is_empty() && execute.len() > 1 {
                    execute_query(&session, execute).await;
                }
            }
        } else {
            // Interactive Shell mode
            let mut query = String::new();
            print!("cqlsh> ");
            loop {
                std::io::stdout().flush().expect("flush failed!");
                let input: String = text_io::read!("{}\n");
                query.push_str(&input);
                if input.to_lowercase().as_str() == "exit" {
                    break;
                }
                if input.ends_with(";") {
                    execute_query(&session, &query).await;
                    query = String::new();
                    print!("cqlsh> ");
                } else {
                    query.push(' ');
                    print!("   ... ");
                }
            }
        }
    }

    Ok(())
}
