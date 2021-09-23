use std::{env, fs::File, io::{prelude::*, self, Write}, iter::Peekable, path::Path, str::FromStr};
use std::process::{Child, Command, Stdio};
use debug_print::debug_println;

enum ErrCode {
    Success = 0,
    Error = 1,
}

#[derive(Debug)]
struct Cmd {
    line: String,
    keyword: String, 
    args: Vec<String>,
    prev_cmd: Option<Child>, 
}

#[derive(Debug)]
struct Cmds {
    splits: Vec<Cmd>,
}
// Builtins {{{
enum Builtin {
    Echo,
    History,
    Cd,
    Pwd,
}

fn builtin_echo (args: &Vec<String>) -> ErrCode {
    println!("{}", args.join(" ")); ErrCode::Success
}

fn builtin_history (_args: &Vec<String>) -> ErrCode {
    let histfile = ".shadohist";
    let home_dir = match home::home_dir() {
        Some(path) => path.display().to_string(),
        None => "/".to_string(),
    };
    let histfile = format!("{}/{}",home_dir,histfile);

    let path = Path::new(&*histfile);
    let mut file = match File::open(&path) {
        Err(why) => { println!("couldn't open {}: {}", path.display(), why); return ErrCode::Error },
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => { println!("couldn't read {}: {}", path.display(), why); return ErrCode::Error },
        Ok(_) => println!("{}", s),
    }
    ErrCode::Success
}

fn builtin_cd (args: &Vec<String>) -> ErrCode {
    let def = match home::home_dir() {
        Some(path) => path.display().to_string(),
        None => "/".to_string(),
    };
    let new_dir = args.iter().peekable().peek().map_or(&def, |x| *x);
    let root = Path::new(new_dir);
    if let Err(e) = env::set_current_dir(&root) { eprintln!("{}", e); return ErrCode::Error; }
    ErrCode::Success
}

fn builtin_pwd (_args: &Vec<String>) -> ErrCode {
    let path = env::current_dir().unwrap();
    println!("{}", path.display());
    ErrCode::Success
}

impl FromStr for Builtin {
    type Err = ();
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(Builtin::Echo),
            "history" => Ok(Builtin::History),
            "cd" => Ok(Builtin::Cd),
            "pwd" => Ok(Builtin::Pwd),
            _ => Err(()),
        }
    }
}
// }}}
fn process_cmd (c: Cmd) -> ErrCode {
    match Builtin::from_str(&c.keyword) {
        Ok(Builtin::Echo) => builtin_echo(&c.args),
        Ok(Builtin::History) => builtin_history(&c.args),
        Ok(Builtin::Cd) => builtin_cd(&c.args),
        Ok(Builtin::Pwd) => builtin_pwd(&c.args),
        _ => { 
            let mut parts = c.line.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;
            let child = Command::new(command).args(args).spawn();
            match child {
                Ok(mut child) => { child.wait().ok(); },
                Err(e) => { eprintln!("{}", e); return ErrCode::Error },
            };
            ErrCode::Success
        }
        // _ => { println!("{}: command not found", &c.keyword); ErrCode::Error },
    }
}

fn print_prompt () {
    let prompt_char = ">";

    print!("{0} ", prompt_char);
    io::stdout().flush().unwrap();
}

fn read_command () -> String {
    let mut command = String::new();

    io::stdin().read_line(&mut command).unwrap(); //.expect("Failed to read in command");
    debug_println!("DEBUG (Raw Input): {:?}", command);
    command
}

fn tokenize_commands (c: String) -> Cmds {
    let mut commands = c.trim().split(" | ").peekable();

    let mut v: Vec<Cmd> = Vec::new();
    while let Some(command) = commands.next() {
        v.push(Cmd::new(command.to_string()));
    }

    debug_println!("DEBUG (Split Commands): {:?}", v);
    Cmds { splits: v }
}

// fn tokenize_command (c: String) -> Cmd {
//     let mut cmd_split: Vec<String> = c.trim().split_whitespace().map(|s| s.to_string()).collect();
//     debug_println!("DEBUG (Split Input): {:?}", cmd_split);
//     println!("c: {}", c);

//     let cmd = Cmd {
//         line: c,
//         keyword: cmd_split.remove(0),
//         args: cmd_split,
//     };
//     debug_println!("DEBUG (Keyword): {:?}", cmd.keyword);
//     debug_println!("DEBUG (Num of Args): {0:?}\nDEBUG (Args): {1:?}", cmd.args.len(), cmd.args);
//     cmd
// }

impl Cmd {
    pub fn new (c: String) -> Cmd {
        let mut cmd_split: Vec<String> = c.trim().split_whitespace().map(|s| s.to_string()).collect();
        Cmd {
            line: c,
            keyword: cmd_split.remove(0),
            args: cmd_split,
            prev_cmd: None,
        }
    }
}

fn main () {
    loop {
        print_prompt();
        let command = read_command();
        if command == "\n" { continue; }
        if command == "exit\n" { return; }
        
        /* TODO: Add history save */

        let cmds = tokenize_commands(command);
        // let cmds = tokenize_command(command);
        // process_cmd(cmds);
    }
}
// Tests {{{
#[cfg(test)]
mod test_tokenize_command {
    use super::*;

    #[test]
    #[ignore]
    fn empty_cmd () {
        assert_eq!("", tokenize_command("test".to_string()).keyword);
    }

    #[test]
    fn test_keyword () {
        assert_eq!("test", tokenize_command("test".to_string()).keyword);
    }

    #[test]
    fn no_args () {
        assert_eq!(0, tokenize_command("test".to_string()).args.len());
    }

    #[test]
    fn one_arg () {
        assert_eq!(1, tokenize_command("test one".to_string()).args.len());
    }

    #[test]
    fn multi_args () {
        assert_eq!(3, tokenize_command("test one two three".to_string()).args.len());
    }

    #[test]
    #[ignore]
    fn quotes () {
        assert_eq!(2, tokenize_command("test \"one two\" three".to_string()).args.len());
    }
}
/* }}} */
