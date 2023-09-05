// Simple test of readline

#include <iostream>
#include <fstream>

#include <readline/readline.h>
#include <readline/history.h>

#include <gqlite.h>

#include "../src/string.h"

using namespace std;

void print_help()
{
  std::cout << R"VGG(.help             show this message
.once ?FILE?      save result of next query in FILE.
.open ?FILE?      close existing connection and reopen FILE.
.quit             Exit this program)VGG"  <<std::endl;
}

int main(int argc, const char** argv)
{
  gqlite::connection handle;
  if(argc == 2)
  {
    handle = gqlite::connection:: create_from_sqlite_file(argv[1]);
  }
  std::cout << "Enter '.help' for usage hints." << std::endl;
  char *line;
  std::string statement;
  std::string output_filename;
  while ((line = readline("gqlite> ")) != nullptr)
  {
    if (*line)
    {
      add_history(line);
      std::string command = line;
      free(line);
      if(command[0] == '.')
      {
        std::vector<std::string> command_split = gqlite::string::split(command, ' ');
        if(command_split[0] == ".help")
        {
          print_help();
        } else if(command_split[0] == ".once")
        {
          if(command_split.size() != 2)
          {
            std::cout << ".once expect a filename as argument" << std::endl;
          } else {
            output_filename = command_split[1];
          }
        } else if(command_split[0] == ".open")
        {
          if(command_split.size() != 2)
          {
            std::cout << ".open expect a filename as argument" << std::endl;
          } else {
            handle = gqlite::connection:: create_from_sqlite_file(command_split[1]);
          }
        } else if(command_split[0] == ".quit")
        {
          return 0;
        } else {
          std::cout << "Unknown command '" << command[0] << "' use '.help' for usage hints." << std::endl;
        }
      } else {
        while(*std::prev(command.end()) != ';' and (line = readline("   ...> ")) != nullptr)
        {
          if (*line)
          {
            add_history(line);
            command += line;
            free(line);
          }          
        }
        *std::prev(command.end()) = ' '; // remove the trailing ';'
        try
        {
          gqlite::value val = handle.execute_oc_query(command);
          if(output_filename.empty())
          {
            std::cout << val.to_json() << std::endl;
          } else {
            std::ofstream ofss;
            ofss.open(output_filename, std::ios::out);
            if(ofss.is_open())
            {
              ofss << val.to_json();
              std::cout << "Results was written to '" << output_filename << "'" << std::endl;
            } else {
              std::cout << "Failed to open '" << output_filename << "' for writting." << std::endl; 
            }
          }
        } catch(const gqlite::exception& _ex)
        {
          std::cout << "Error occured during execution of query: '" << _ex.what() << "'" << std::endl;
        }
      }
    } else {
      std::cout << "Enter '.help' for usage hints." << std::endl;
    }
  }

  return 0;
}
