fn print_help()
{
  println!(
    "List of commands:
.once ?FILE?      save result of next query in FILE.
.open ?FILE?      close existing connection and reopen FILE.
.quit             Exit this program
.help             show this message

To execute a query, write the query and end it with a ';'"
  );
}

fn main_loop(rl: &mut rustyline::DefaultEditor) -> rustyline::Result<()>
{
  let mut connection: Option<gqlitedb::Connection> = None;
  let mut args = std::env::args();
  args.next(); // remove program name
  if let Some(filename) = args.next()
  {
    let connection_res = gqlitedb::Connection::open(filename, gqlitedb::ValueObject::new());
    match connection_res
    {
      Ok(c) =>
      {
        connection = Some(c);
      }
      Err(msg) =>
      {
        println!("{:?}", msg);
      }
    }
  }
  loop
  {
    let line = rl.readline("gqlite> ")?;
    if line.len() > 0
    {
      if line.starts_with(".")
      {
        rl.add_history_entry(line.as_str())?;
        let splited_line: Vec<_> = line.split(" ").collect();
        match splited_line[0]
        {
          ".help" =>
          {
            print_help();
          }
          ".open" =>
          {
            if line.len() < 2
            {
              println!("Missing argument to .open");
            }
            let connection_res =
              gqlitedb::Connection::open(splited_line[1], gqlitedb::ValueObject::new());
            match connection_res
            {
              Ok(c) =>
              {
                connection = Some(c);
              }
              Err(msg) =>
              {
                println!("{:?}", msg);
              }
            }
          }
          ".quit" =>
          {
            return Ok(());
          }
          ".once" =>
          {
            println!("'.once' is not implemented yet.");
          }
          unknown_command =>
          {
            println!(
              "Unknown command '{}', use '.help' to see the list of commands.",
              unknown_command
            );
          }
        }
      }
      else
      {
        let mut query = line;
        while !query.ends_with(";")
        {
          let readline = rl.readline("   ...> ");
          match readline
          {
            Ok(line) =>
            {
              if line.len() == 0
              {
                break;
              }
              else
              {
                query += "\n";
                query += &line;
              }
            }
            Err(rustyline::error::ReadlineError::Interrupted) =>
            {
              query = String::new();
              break;
            }
            Err(other_error) =>
            {
              return Err(other_error);
            }
          }
        }
        if query.len() > 0
        {
          rl.add_history_entry(query.as_str())?;
          match connection
          {
            Some(ref c) =>
            {
              let qr = c.execute_query(query, gqlitedb::ValueObject::new());
              match qr
              {
                Ok(value) =>
                {
                  match value
                  {
                    gqlitedb::Value::Array(arr) =>
                    {
                      let mut builder = tabled::builder::Builder::new();
                      // let mut it = arr.iter();
                      // let keys = it.next().map_or(Vec::<String>::new(), |v| {
                      //   match v {
                      //     gqlitedb::Value::Array(arr) => arr.iter().map(|x| x.to_string()).collect(),
                      //     _ => vec![]
                      //   }
                      // });
                      // builder.push_record(keys.iter());
                      // it.for_each(|row| {
                      //   match row {
                      //     gqlitedb::Value::Object(obj) => {
                      //       builder.push_record(keys.iter().map(|k| obj[k].to_string()));
                      //     },
                      //     _ => {
                      //       println!("Unexpected: {}", row);
                      //     }
                      //   }
                      // });
                      arr.iter().for_each(|row| match row
                      {
                        gqlitedb::Value::Array(arr) =>
                        {
                          builder.push_record(arr.iter().map(|x| x.to_string()))
                        }
                        _ =>
                        {
                          println!("Unexpected: {}", row);
                        }
                      });

                      let table = builder
                        .build()
                        .with(tabled::settings::Style::ascii_rounded())
                        .to_string();
                      println!("{}", table);
                    }
                    _ =>
                    {}
                  }
                }
                Err(err) => match err.error()
                {
                  gqlitedb::Error::CompileTime(ct) =>
                  {
                    println!("Compilation error:\n{}", ct.to_string());
                  }
                  _ =>
                  {
                    println!("Query execution failed: {:?}", err);
                  }
                },
              }
            }
            None =>
            {
              println!("No database connection, use '.open' before executing a query.");
            }
          }
        }
      }
    }
  }
}

fn main() -> rustyline::Result<()>
{
  // `()` can be used when no completer is required
  let mut rl = rustyline::DefaultEditor::new()?;
  let gqlite_history = standard_paths::StandardPaths::new("gqlitecli", "gqlite.org")
    .writable_location(standard_paths::LocationType::ConfigLocation)?
    .join("gqlite_history");
  if rl.load_history(&gqlite_history).is_err()
  {}
  println!("Enter '.help' for usage hints.");
  match main_loop(&mut rl)
  {
    Ok(_) =>
    {}
    Err(rustyline::error::ReadlineError::Interrupted) =>
    {
      println!("CTRL-C");
    }
    Err(rustyline::error::ReadlineError::Eof) =>
    {
      println!("CTRL-D");
    }
    Err(err) =>
    {
      println!("Error: {:?}", err);
    }
  }
  rl.save_history(&gqlite_history)?;
  Ok(())
}
