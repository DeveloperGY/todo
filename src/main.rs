use std::{ffi::OsStr, fmt::Display, fs::DirEntry, io::Write, path::PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Note {
    text: String,
    checked: bool,
}

impl Note {
    fn check(&mut self) {
        self.checked = true;
    }

    fn uncheck(&mut self) {
        self.checked = false;
    }

    fn is_checked(&self) -> bool {
        self.checked
    }
}

impl<T> From<T> for Note
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let text = value.as_ref().trim().to_string();
        Self {
            text,
            checked: false,
        }
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let check = if self.checked { '#' } else { ' ' };

        let mut buf = String::new();
        for (i, line) in self.text.trim().lines().enumerate() {
            if i != 0 {
                buf.push_str("      ");
            }
            buf.push_str(line);
            buf.push('\n');
        }

        write!(f, "[{}] - {}", check, buf.trim())
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct List {
    notes: Vec<Note>,
}

impl List {
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn check_note(&mut self, note_index: usize) {
        if self.notes.len() > note_index {
            self.notes[note_index].check();
        }
    }

    pub fn check_all(&mut self) {
        for note in &mut self.notes {
            note.check();
        }
    }

    pub fn uncheck_note(&mut self, note_index: usize) {
        if self.notes.len() > note_index {
            self.notes[note_index].uncheck();
        }
    }

    pub fn uncheck_all(&mut self) {
        for note in &mut self.notes {
            note.uncheck();
        }
    }

    pub fn remove_all(&mut self) {
        self.notes.clear();
    }

    pub fn remove_checked(&mut self) {
        self.notes = self
            .notes
            .iter()
            .filter(|note| !note.is_checked())
            .cloned()
            .collect::<Vec<_>>();
    }

    pub fn remove_unchecked(&mut self) {
        self.notes = self
            .notes
            .iter()
            .filter(|note| note.is_checked())
            .cloned()
            .collect::<Vec<_>>();
    }

    pub fn remove_note(&mut self, note_index: usize) {
        if self.notes.len() > note_index {
            self.notes.remove(note_index);
        }
    }
}

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.notes.len() {
            0 => write!(f, ""),
            _ => {
                let mut buf = String::new();

                let digit_count = self.notes.len().ilog10() + 1;

                for (note_num, note) in self.notes.iter().enumerate() {
                    let note_string = note.to_string();
                    let mut lines = note_string.lines();

                    buf.push_str(&format!("{} {}\n", note_num + 1, lines.next().unwrap()));

                    lines.for_each(|line| {
                        buf.push_str(&(0..=digit_count).map(|_| ' ').collect::<String>());
                        buf.push_str(&format!("{}\n", line));
                    });
                }

                write!(f, "{}", buf.trim())
            }
        }
    }
}

fn todo_dir_path() -> Option<PathBuf> {
    std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|_| default_data_home())
        .map(|path| path.join("fp_todo"))
        .ok()
}

fn default_data_home() -> Result<PathBuf, std::env::VarError> {
    std::env::var("HOME").map(|home| PathBuf::from(&home).join(".local/share"))
}

fn main() {
    // Ensure data storage directory exists
    if todo_dir_path()
        .and_then(|path| {
            if !path.exists() {
                std::fs::create_dir(path).ok()
            } else {
                Some(())
            }
        })
        .is_none()
    {
        eprintln!("Failed to ensure todo list storage path exists!");
        return;
    }

    // Retrieve arguments
    let arg_strings = std::env::args().skip(1).collect::<Vec<_>>();
    let arg_strs = arg_strings
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>();

    // Execute requested command
    match arg_strs[..] {
        ["list" | "-l", ref args @ ..] => list(args),
        ["note" | "-n", ref args @ ..] => note(args),
        _ => (),
    };
}

fn note(args: &[&str]) {
    match args {
        ["add", list_name, note @ ..] => note_add(list_name, note),
        ["remove", list_name, note_indices @ ..] => note_remove(list_name, note_indices),
        ["check", list_name, note_indices @ ..] => note_check(list_name, note_indices),
        ["uncheck", list_name, note_indices @ ..] => note_uncheck(list_name, note_indices),
        _ => (),
    };
}

fn note_add(list_name: &str, note: &[&str]) {
    if todo_dir_path()
        .map(|path| path.join(format!("{}.todo", list_name)))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|data| ron::from_str::<List>(&data).ok())
        .map(|mut list| {
            let note = note.join(" ").into();
            list.add_note(note);
            list
        })
        .and_then(|list| {
            let mut file = std::fs::File::options()
                .write(true)
                .truncate(true)
                .open(todo_dir_path().unwrap().join(format!("{}.todo", list_name)))
                .ok()?;

            file.write_all(ron::to_string(&list).unwrap().as_bytes())
                .ok()
        })
        .is_none()
    {
        eprintln!("Error: Failed to add note to todo list!");
    }
}

fn note_check(list_name: &str, note_indices: &[&str]) {
    if todo_dir_path()
        .map(|path| path.join(format!("{}.todo", list_name)))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|data| ron::from_str::<List>(&data).ok())
        .map(|mut list| {
            match note_indices {
                ["all"] => list.check_all(),
                note_index_strs => {
                    note_index_strs
                        .iter()
                        .flat_map(|note_index_str| {
                            let res = note_index_str.parse::<usize>().ok();

                            if res.is_none() {
                                println!(
                                    "\'{}\' did not parse into a valid note index",
                                    note_index_str
                                )
                            }

                            res
                        })
                        .filter(|index| *index != 0)
                        .for_each(|index| list.check_note(index - 1));
                }
            }

            list
        })
        .and_then(|list| {
            let mut file = std::fs::File::options()
                .write(true)
                .truncate(true)
                .open(todo_dir_path().unwrap().join(format!("{}.todo", list_name)))
                .ok()?;

            file.write_all(ron::to_string(&list).unwrap().as_bytes())
                .ok()
        })
        .is_none()
    {
        eprintln!("Error: Failed to check note(s)!");
    }
}

fn note_uncheck(list_name: &str, note_indices: &[&str]) {
    if todo_dir_path()
        .map(|path| path.join(format!("{}.todo", list_name)))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|data| ron::from_str::<List>(&data).ok())
        .map(|mut list| {
            match note_indices {
                ["all"] => list.uncheck_all(),
                note_index_strs => {
                    note_index_strs
                        .iter()
                        .flat_map(|note_index_str| {
                            let res = note_index_str.parse::<usize>().ok();

                            if res.is_none() {
                                println!(
                                    "\'{}\' did not parse into a valid note index",
                                    note_index_str
                                )
                            }

                            res
                        })
                        .filter(|index| *index != 0)
                        .for_each(|index| list.uncheck_note(index - 1));
                }
            }

            list
        })
        .and_then(|list| {
            let mut file = std::fs::File::options()
                .write(true)
                .truncate(true)
                .open(todo_dir_path().unwrap().join(format!("{}.todo", list_name)))
                .ok()?;

            file.write_all(ron::to_string(&list).unwrap().as_bytes())
                .ok()
        })
        .is_none()
    {
        eprintln!("Error: Failed to uncheck note(s)!");
    }
}

fn note_remove(list_name: &str, note_indices: &[&str]) {
    if todo_dir_path()
        .map(|path| path.join(format!("{}.todo", list_name)))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|data| ron::from_str::<List>(&data).ok())
        .map(|mut list| {
            match note_indices {
                ["all"] => list.remove_all(),
                ["checked"] => list.remove_checked(),
                ["unchecked"] => list.remove_unchecked(),
                note_index_strs => {
                    let mut note_indices: Vec<usize> = note_index_strs
                        .iter()
                        .flat_map(|note_index_str| {
                            let res = note_index_str.parse::<usize>().ok();

                            if res.is_none() {
                                println!(
                                    "\'{}\' did not parse into a valid note index",
                                    note_index_str
                                )
                            }

                            res
                        })
                        .filter(|index| *index != 0)
                        .collect();

                    // sort + rev is necessary to ensure that note indices still refer to the
                    // intended notes
                    note_indices.sort();

                    note_indices.into_iter().rev().for_each(|index| {
                        list.remove_note(index - 1);
                    });
                }
            }

            list
        })
        .and_then(|list| {
            let mut file = std::fs::File::options()
                .write(true)
                .truncate(true)
                .open(todo_dir_path().unwrap().join(format!("{}.todo", list_name)))
                .ok()?;

            file.write_all(ron::to_string(&list).unwrap().as_bytes())
                .ok()
        })
        .is_none()
    {
        eprintln!("Error: Failed to remove note(s)!");
    }
}

fn list(args: &[&str]) {
    match args {
        [] => list_lists(),
        ["new", name] => list_new(name),
        ["del", name] => list_del(name),
        ["read", name] => list_read(name),
        [inv, ..] => println!("Invalid command: {}", inv),
    };
}

fn list_lists() {
    if todo_dir_path()
        .and_then(|dir_path| std::fs::read_dir(dir_path).ok())
        .map(|entries| {
            for entry in entries.flatten().map(|e| e.path()) {
                if entry
                    .extension()
                    .and_then(|ext| ext.to_str().map(|ext| ext.to_string()))
                    .is_some()
                {
                    if let Some(list_name) = entry
                        .file_stem()
                        .and_then(|stem| stem.to_str().map(|s| s.to_string()))
                    {
                        println!("{}", list_name);
                    }
                }
            }
        })
        .is_none()
    {
        eprintln!("Failed to create new list!");
    }
}

fn list_new(name: &str) {
    if todo_dir_path()
        .map(|path| path.join(format!("{}.todo", name)))
        .and_then(|path| std::fs::File::create_new(path).ok())
        .and_then(|mut file| {
            let empty_list = List::default();
            file.write_all(ron::to_string(&empty_list).ok()?.as_bytes())
                .ok()
        })
        .is_none()
    {
        eprintln!("Failed to create new list!");
    }
}

fn list_del(name: &str) {
    if todo_dir_path()
        .map(|path| path.join(format!("{}.todo", name)))
        .and_then(|path| std::fs::remove_file(path).ok())
        .is_none()
    {
        eprintln!("Failed to delete todo list!")
    }
}

fn list_read(name: &str) {
    if todo_dir_path()
        .map(|path| path.join(format!("{}.todo", name)))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|data| ron::from_str::<List>(&data).ok())
        .map(|list| println!("{}", list))
        .is_none()
    {
        eprintln!("Failed to read todo list!");
    }
}
