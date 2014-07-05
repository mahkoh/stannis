pub enum Command {
    Quit,
    Test,
}

#[deriving(Show)]
pub enum Error {
    EmptyCommand,
    UnknownCommand,
}

impl Command {
    fn first() -> Command {
        Quit
    }

    fn next(self) -> Option<Command> {
        match self {
            Quit => Some(Test),
            Test => None,
        }
    }

    fn text(self) -> &'static str {
        match self {
            Quit => "q",
            Test => "test",
        }
    }
}

pub fn parse(s: &str) -> Result<Command, Error> {
    let iter = s.words();
    let first = match iter.next() {
        Some(s) => s,
        None => return Err(EmptyCommand),
    };
    let mut command = Command::first();
    loop {
        if first == command.text() {
            return Ok(command);
        }
        command = match command.next() {
            Some(c) => c,
            None => break,
        };
    }
    Err(UnknownCommand)
}
