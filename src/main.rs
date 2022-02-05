use std::borrow::Cow;
use clap::{ErrorKind, IntoApp, Parser};
use scylla::{Session, SessionBuilder};
use std::io::{Error, Write};
use scylla::frame::response::result::CqlValue;
use std::time::Duration;


#[derive(Parser, Debug)]
#[clap(about = "I am a cqlsh alternative, just pass `-h`")]
struct Cli {
    host: String,

    #[clap(long, short, help = "Execute the given statement, then exit")]
    execute: Option<String>,

    #[clap(long, short, help = "Username to authenticate against Cassandra with")]
    user: Option<String>,

    #[clap(long, short, help = "Password to authenticate against Cassandra with, should be used in conjunction with --user")]
    password: Option<String>,

    #[clap(long, short, help = "Keyspace to authenticate to")]
    keyspace: Option<String>,

    #[clap(long, default_value = "2000", help = "Specify the connection timeout in seconds (defaults to 2s)")]
    connect_timeout: u64
}

fn fmt_col(col: &CqlValue) -> Cow<str> {
    match col {
        CqlValue::Ascii(col) => {
            Cow::Borrowed(col)
        }
        CqlValue::Boolean(col) => {
            Cow::Borrowed(if *col {
                "true"
            } else {
                "false"
            })
        }
        CqlValue::Blob(_) => {
            Cow::Borrowed("<blob>")
        }
        CqlValue::Counter(col) => {
            Cow::Owned(col.0.to_string())
        }
        CqlValue::Decimal(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Date(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Double(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Empty => {
            Cow::Borrowed("")
        }
        CqlValue::Float(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Int(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::BigInt(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Text(col) => {
            Cow::Borrowed(col)
        }
        CqlValue::Timestamp(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Inet(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::List(col) => {
            let mut out = String::new();
            out.push('{');
            for value in col {
                out.push_str(&*match value {
                    CqlValue::List(_) => {
                        fmt_col(value)
                    }
                    CqlValue::Map(_) => {
                        fmt_col(value)
                    }
                    CqlValue::Set(_) => {
                        fmt_col(value)
                    }
                    CqlValue::UserDefinedType{..} => {
                        fmt_col(value)
                    }
                    _ => {
                        Cow::Owned(format!("'{}'", fmt_col(value)))
                    }
                });
                if value != col.last().unwrap() {
                    out.push_str(", ");
                }
            }
            out.push('}');

            Cow::Owned(out)
        }
        CqlValue::Map(col) => {
            let mut out = String::new();
            out.push('{');
            for (key, value) in col {
                out.push_str(&*format!("{}: {}", match key {
                    CqlValue::List(_) => {
                        fmt_col(key)
                    }
                    CqlValue::Map(_) => {
                        fmt_col(key)
                    }
                    CqlValue::Set(_) => {
                        fmt_col(key)
                    }
                    CqlValue::UserDefinedType{..} => {
                        fmt_col(key)
                    }
                    _ => {
                        Cow::Owned(format!("'{}'", fmt_col(key)))
                    }
                }, match value {
                    CqlValue::List(_) => {
                        fmt_col(value)
                    }
                    CqlValue::Map(_) => {
                        fmt_col(value)
                    }
                    CqlValue::Set(_) => {
                        fmt_col(value)
                    }
                    CqlValue::UserDefinedType{..} => {
                        fmt_col(value)
                    }
                    _ => {
                        Cow::Owned(format!("'{}'", fmt_col(value)))
                    }
                }));
                if value != &col.last().unwrap().0 {
                    out.push_str(", ");
                }
            }
            out.push('}');

            Cow::Owned(out)
        }
        CqlValue::Set(col) => {
            let mut out = String::new();
            out.push('{');
            for value in col {
                out.push_str(&*match value {
                    CqlValue::List(_) => {
                        fmt_col(value)
                    }
                    CqlValue::Map(_) => {
                        fmt_col(value)
                    }
                    CqlValue::Set(_) => {
                        fmt_col(value)
                    }
                    CqlValue::UserDefinedType{..} => {
                        fmt_col(value)
                    }
                    _ => {
                        Cow::Owned(format!("'{}'", fmt_col(value)))
                    }
                });
                if value != col.last().unwrap() {
                    out.push_str(", ");
                }
            }
            out.push('}');

            Cow::Owned(out)
        }
        CqlValue::UserDefinedType { fields, .. } => {
            let mut out = String::new();
            out.push('{');
            for (key, value) in fields {
                out.push_str(&*match value {
                    Some(value) => {
                        match value {
                            CqlValue::List(_) => {
                                format!("{}: {}", key, fmt_col(value))
                            }
                            CqlValue::Map(_) => {
                                format!("{}: {}", key, fmt_col(value))
                            }
                            CqlValue::Set(_) => {
                                format!("{}: {}", key, fmt_col(value))
                            }
                            CqlValue::UserDefinedType{..} => {
                                format!("{}: {}", key, fmt_col(value))
                            }
                            _ => {
                                format!("{}: '{}'", key, fmt_col(value))
                            }
                        }
                    }
                    None => {
                        format!("{}: null", key)
                    }
                });
                if key != &fields.last().unwrap().0 {
                    out.push_str(", ");
                }
            }
            out.push('}');

            Cow::Owned(out)
        }
        CqlValue::SmallInt(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::TinyInt(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Time(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Timeuuid(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Tuple(col) => {
            let mut out = String::new();
            out.push('{');
            for value in col {
                out.push_str(&*match value {
                    Some(value) => {
                        match value {
                            CqlValue::List(_) => {
                                fmt_col(value)
                            }
                            CqlValue::Map(_) => {
                                fmt_col(value)
                            }
                            CqlValue::Set(_) => {
                                fmt_col(value)
                            }
                            CqlValue::UserDefinedType{..} => {
                                fmt_col(value)
                            }
                            _ => {
                                Cow::Owned(format!("'{}'", fmt_col(value)))
                            }
                        }
                    }
                    None => {
                        Cow::Owned(String::from("null"))
                    }
                });
                let last = col.last().unwrap();
                if value.is_none() && last.is_none() || value.as_ref().unwrap() == last.as_ref().unwrap()  {
                    out.push_str(", ");
                }
            }
            out.push('}');

            Cow::Owned(out)
        }
        CqlValue::Uuid(col) => {
            Cow::Owned(col.to_string())
        }
        CqlValue::Varint(col) => {
            Cow::Owned(col.to_string())
        }
    }
}

async fn execute_query(session: &Session, query: &str) {
    match session.query(query, &[]).await {
        Ok(query_result) => {
            let mut width: Vec<usize> = query_result.col_specs.iter()
                .map(|spec| spec.name.chars().count()).collect();

            let col_specs = &query_result.col_specs;
            let alt_row = Vec::new();
            let rows = query_result.rows.as_ref().unwrap_or(&alt_row);

            if rows.len() > 0 {
                for col_spec in rows {
                    for (col, w) in col_spec.columns.iter().zip(width.iter_mut()) {
                        if let Some(col) = col {
                            let s = fmt_col(col);
                            let l = s.chars().count();
                            if l > *w {
                                *w = l;
                            }
                        } else {
                            if 4 > *w {
                                *w = 4;
                            }
                        }
                    }
                }
                println!();
                for (col_spec, w) in col_specs.iter().zip(width.iter()) {
                    print!("| {}", col_spec.name);
                    for _ in 0..(w - col_spec.name.chars().count()) {
                        print!(" ");
                    }
                    print!(" ");
                }
                print!(" |\n");
                for (_, w) in col_specs.iter().zip(width.iter()) {
                    print!("+-");
                    for _ in 0..*w {
                        print!("-");
                    }
                    print!("-");
                }
                print!("-+\n");
                for col_spec in rows {
                    for (col, w) in col_spec.columns.iter().zip(width.iter()) {
                        if let Some(col) = col {
                            let s = fmt_col(col);
                            print!("| {}", s);
                            for _ in 0..(w - s.chars().count()) {
                                print!(" ");
                            }
                            print!(" ");
                        } else {
                            print!("| null");
                            for _ in 0..(w - 4) {
                                print!(" ");
                            }
                            print!(" ");
                        }
                    }
                    print!(" |\n");
                }
                println!();
            } else {
                println!();
                println!("Empty Result Set");
                println!();
            }
        }
        Err(err) => {
            println!("\n{}\n", err);
        }
    }
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

    let mut session_builder = SessionBuilder::new();
    session_builder = session_builder.known_node(args.host);
    if args.user.is_some() && args.password.is_some() {
        session_builder = session_builder.user(args.user.unwrap(), args.password.unwrap());
    }
    if args.keyspace.is_some() {
        session_builder = session_builder.use_keyspace(args.keyspace.unwrap(), false);
    }
    session_builder = session_builder.connection_timeout(Duration::from_millis(args.connect_timeout));

    let session: Session = session_builder.build().await
        .expect("could not connect to database");

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
            if input == "exit" {
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

    Ok(())
}
