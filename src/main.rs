use std::{env, fs::File, io::{prelude::*, self, Write}, path::Path, str::FromStr};
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
}

#[derive(Debug)]
struct Cmds {
    line: String,
    splits: Vec<Cmd>,
}

impl Cmd {
    pub fn new (c: &str) -> Cmd {
        let mut cmd_split: Vec<String> = c.trim().split_whitespace().map(|s| s.to_string()).collect();
        Cmd {
            line: c.to_string(),
            keyword: cmd_split.remove(0),
            args: cmd_split,
        }
    }
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
fn process_cmd (c: Cmd, some: bool, mut previous_cmd: Option<Child>) -> (ErrCode, Option<Child>) {
    let mut exit_sts = ErrCode::Success;
    match Builtin::from_str(&c.keyword) {
        Ok(Builtin::Echo) => (builtin_echo(&c.args), None),
        Ok(Builtin::History) => (builtin_history(&c.args), None),
        Ok(Builtin::Cd) => (builtin_cd(&c.args), None),
        Ok(Builtin::Pwd) => (builtin_pwd(&c.args), None),
        _ => { 
            println!("TEST: {}", c.keyword);
            let stdin = previous_cmd.map_or(Stdio::inherit(),
                |output: Child| Stdio::from(output.stdout.unwrap()));
            let stdout = if !some { Stdio::piped() } else { Stdio::inherit() };

            let output = Command::new(c.keyword).args(&c.args).stdin(stdin)
                .stdout(stdout).spawn();
            match output {
                Ok(output) => { previous_cmd = Some(output); },
                Err(e) => { previous_cmd = None; eprintln!("{}", e); exit_sts = ErrCode::Error; },
            }
            debug_println!("DEBUG (Previous): {:?}", previous_cmd);
            println!("Prev: {:?}", previous_cmd);
            (exit_sts, previous_cmd)
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

fn tokenize_commands (c: &str) -> Cmds {
    let mut commands = c.trim().split(" | ").peekable();

    let mut v: Vec<Cmd> = Vec::new();
    while let Some(command) = commands.next() {
        v.push(Cmd::new(command));
    }
    Cmds { line: c.to_string(), splits: v }
}


fn main () {
    loop {
        print_prompt();
        let ln = read_command();
        if ln == "\n" { continue; }
        if ln == "exit\n" { return; }
        
        /* TODO: Add history save */

        let cmds = tokenize_commands(&ln);
        let mut ret = (ErrCode::Success, None);
        for (i, cmd) in cmds.splits.into_iter().enumerate() {
            ret = process_cmd(cmd, true, /*if i != 0 {true} else {false},*/ ret.1);
        };
        if let Some(mut final_command) = ret.1 {
            final_command.wait().ok();
        }
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
