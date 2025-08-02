#![deny(warnings)]

use std::{
  fs,
  io::{self, BufRead},
};

fn print_help()
{
  println!(
    "List of commands:
.help             Show this message.
.once ?FILE?      Save result of next query in FILE.
.open ?FILE?      Close existing connection and reopen FILE.
.quit             Exit this program.
.read ?FILE?      Read input from the command line.

To execute a query, write the query and end it with a ';'"
  );
}

fn print_results(arr: &Vec<gqlitedb::Value>)
{
  let mut builder = tabled::builder::Builder::new();
  arr.iter().for_each(|row| match row
  {
    gqlitedb::Value::Array(arr) => builder.push_record(arr.iter().map(|x| x.to_string())),
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

//   ____ _ _ ___ _                 _
//  / ___| (_)_ _| |_ ___ _ __ __ _| |_ ___  _ __
// | |   | | || || __/ _ \ '__/ _` | __/ _ \| '__|
// | |___| | || || ||  __/ | | (_| | || (_) | |
//  \____|_|_|___|\__\___|_|  \__,_|\__\___/|_|

trait CliIterator<E>: Iterator<Item = Result<String, E>>
{
  fn add_history_entry(&mut self, _: &String) -> Result<(), E>
  {
    Ok(())
  }
}

impl CliIterator<std::io::Error> for io::Lines<io::BufReader<fs::File>> {}

struct ReadLineIterator<'de>
{
  rl: &'de mut rustyline::DefaultEditor,
  first: bool,
}

impl<'de> Iterator for ReadLineIterator<'de>
{
  type Item = rustyline::Result<String>;
  fn next(&mut self) -> Option<Self::Item>
  {
    let line = if self.first
    {
      self.first = false;
      self.rl.readline("gqlite> ")
    }
    else
    {
      self.rl.readline("   ...> ")
    };

    match line
    {
      Ok(line) => Some(Ok(line)),
      Err(rustyline::error::ReadlineError::Interrupted) => None,
      Err(other_error) => Some(Err(other_error)),
    }
  }
}

impl<'de> CliIterator<rustyline::error::ReadlineError> for ReadLineIterator<'de>
{
  fn add_history_entry(&mut self, h: &String) -> Result<(), rustyline::error::ReadlineError>
  {
    self.rl.add_history_entry(h).map(|_| ())
  }
}

//   ____ _ _
//  / ___| (_)
// | |   | | |
// | |___| | |
//  \____|_|_|

struct Cli
{
  connection: Option<gqlitedb::Connection>,
}

impl Cli
{
  fn process_lines<E>(
    &mut self,
    mut it: impl CliIterator<E>,
    exhaust_iterator: bool,
  ) -> Result<bool, E>
  {
    loop
    {
      let line = it.next().transpose()?;
      let line = match line
      {
        Some(line) => line,
        None => return Ok(false),
      };
      if line.len() > 0
      {
        if line.starts_with(".")
        {
          it.add_history_entry(&line)?;
          let splited_line: Vec<_> = line.split(" ").collect();
          match splited_line[0]
          {
            ".help" =>
            {
              print_help();
            }
            ".open" =>
            {
              if splited_line.len() < 2
              {
                println!("Missing argument to .open");
              }
              else
              {
                let connection_res = gqlitedb::Connection::builder()
                  .path(splited_line[1])
                  .create();
                match connection_res
                {
                  Ok(c) =>
                  {
                    self.connection = Some(c);
                  }
                  Err(msg) =>
                  {
                    println!("{:?}", msg);
                  }
                }
              }
            }
            ".quit" =>
            {
              return Ok(true);
            }
            ".once" =>
            {
              println!("'.once' is not implemented yet.");
            }
            ".read" =>
            {
              if splited_line.len() < 2
              {
                println!("Missing argument to .read");
              }
              else
              {
                let file = fs::File::open(splited_line[1]);
                match file
                {
                  Ok(file) =>
                  {
                    let lines = io::BufReader::new(file).lines();
                    if let Err(e) = self.process_lines(lines, true)
                    {
                      println!(
                        "Error when processing file '{:?}': {:?}",
                        splited_line[1], e
                      );
                    }
                  }
                  Err(e) =>
                  {
                    println!(
                      "Read file '{:?}' failed with error '{:?}'.",
                      splited_line[1], e
                    );
                  }
                }
              }
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
            let line = it.next().transpose()?;
            match line
            {
              Some(line) =>
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
              None => break,
            }
          }
          if query.len() > 0
          {
            it.add_history_entry(&query)?;
            match self.connection
            {
              Some(ref c) =>
              {
                let qr = c.execute_query(query, gqlitedb::ValueMap::new());
                match qr
                {
                  Ok(value) => match value
                  {
                    gqlitedb::Value::Array(arr) =>
                    {
                      print_results(&arr);
                    }
                    gqlitedb::Value::Map(map) =>
                    {
                      if matches!(map.get("type"), Some(gqlitedb::Value::String(s)) if s == "results")
                      {
                        map.get("results").map(|results| match results
                        {
                          gqlitedb::Value::Array(arr) =>
                          {
                            for val in arr
                            {
                              match val
                              {
                                gqlitedb::Value::Array(arr) =>
                                {
                                  print_results(arr);
                                }
                                _ =>
                                {}
                              }
                            }
                          }
                          _ =>
                          {}
                        });
                      }
                    }
                    _ =>
                    {}
                  },
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
      if !exhaust_iterator
      {
        return Ok(false);
      }
    }
  }
}

fn main_loop(rl: &mut rustyline::DefaultEditor) -> rustyline::Result<()>
{
  let mut cli = Cli { connection: None };

  let mut args = std::env::args();
  args.next(); // remove program name
  if let Some(filename) = args.next()
  {
    let connection_res = gqlitedb::Connection::builder().path(filename).create();
    match connection_res
    {
      Ok(c) =>
      {
        cli.connection = Some(c);
      }
      Err(msg) =>
      {
        println!("{:?}", msg);
      }
    }
  }

  loop
  {
    if cli.process_lines(ReadLineIterator { rl, first: true }, false)?
    {
      return Ok(());
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
