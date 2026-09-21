extern crate globmatch;
use std::env;
use std::io::{BufWriter, Write, stdout};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use rustyline::Editor;
use rustyline::history::FileHistory;

use crossterm::{
    ExecutableCommand,
    terminal::{Clear, ClearType},
};

const DISPLAY_LIMIT: usize = 100;

pub struct FileFinder {
    stack_vec: Vec<PathBuf>,
    pub extention: String,
    searchword: String,
}

impl FileFinder {
    pub fn new() -> Self {
        Self {
            stack_vec: vec![],
            extention: String::new(),
            searchword: String::new(),
        }
    }
    fn search(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let cur_path = env::current_dir()?;
        item_search(&cur_path, &self.extention, &self.searchword)
    }

    fn display(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut output_buffer = BufWriter::new(stdout());
        // 表示する項目数の制限
        let limitnum = self.stack_vec.len().min(DISPLAY_LIMIT);
        writeln!(output_buffer, "--------\nResults\n--------")?;

        for it in self.stack_vec[..limitnum].iter() {
            writeln!(output_buffer, "{}", it.display())?;
        }

        writeln!(
            output_buffer,
            "extention: {},word: {} => {} hits",
            self.extention,
            self.searchword,
            self.stack_vec.len()
        )?;

        output_buffer.flush()?;
        Ok(())
    }

    pub fn set_extension(
        &self,
        my_readline: &mut Editor<(), FileHistory>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        loop {
            let rline = my_readline.readline("extension >>")?;

            match rline.is_empty() {
                true => println!("Please input search word.. '*' is wildcard."),
                false => {
                    println!("Extension:{}", self.extention);
                    return Ok(rline);
                }
            };
        }
    }

    fn open_file(
        &self,
        my_readline: &mut Editor<(), FileHistory>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let limitnum = 100;

        loop {
            let mut output_buffer = BufWriter::new(stdout());
            for (i, path) in self.stack_vec.iter().enumerate() {
                if i >= limitnum {
                    writeln!(
                        output_buffer,
                        "There are more than {limitnum} applicable items"
                    )?;
                    break;
                }

                writeln!(output_buffer, "{i}:{path:?}")?;
            }

            output_buffer.flush()?;

            let rline = my_readline.readline("select number >>")?;

            match rline.as_str() {
                "q" | "quit" | "@q" | "@quit" => {
                    break;
                }

                _ => match rline.parse::<usize>() {
                    Ok(n) if n < self.stack_vec.len() => {
                        opendir(&self.stack_vec[n])?;
                    }
                    Ok(_) => {
                        println!("Wrong number.");
                    }

                    Err(e) => println!("{e}"),
                },
            }
        }
        Ok(())
    }

    pub fn manage_token(
        &mut self,
        my_readline: &mut Editor<(), FileHistory>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut output_buffer = BufWriter::new(stdout());

        loop {
            writeln!(
                output_buffer,
                "\n\n@ext << change extention, @open << open file, @q or @quit << exit"
            )?;

            output_buffer.flush()?;

            let readline = my_readline.readline(format!("ext:{} =>>", self.extention).as_str())?;

            let in_token = readline.trim();

            output_buffer.execute(Clear(ClearType::All))?;

            match in_token {
                "@quit" | "@q" => break,

                "@ext" | "@e" | "@ex" => {
                    self.extention = self.set_extension(my_readline)?;
                }
                "@open" | "@o" | "@op" => {
                    self.open_file(my_readline)?;
                }
                _ => {
                    let searchword = in_token.to_string();
                    self.searchword = searchword;
                    let start = Instant::now();
                    self.stack_vec = self.search()?;
                    self.display()?;
                    println!("------------------------------------");
                    let end = Instant::now();
                    println!(
                        "Search process took {} ms.",
                        end.duration_since(start).as_millis()
                    );
                }
            }
        }
        Ok(())
    }
}

pub fn item_search(
    rootpath: &PathBuf,
    extension: &str,
    searchword: &str,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let pattern = format!("**/*.{extension}");

    let rootpath = Path::new(rootpath);

    let builder = globmatch::Builder::new(&pattern)
        .case_sensitive(false)
        .build(rootpath)?;

    let result =
        globmatch::IterAll::filter_entry(builder.into_iter(), |p| !globmatch::is_hidden_entry(p))
            .flatten()
            .filter(|x| is_matchword(searchword)(x))
            .collect::<Vec<_>>();

    Ok(result)
}

fn is_matchword<'a>(searchword: &'a str) -> impl Fn(&'a PathBuf) -> bool {
    // x:PathBufの中にsearchwordが含まれるかを判定する関数を返す。部分適用関数
    move |x: &PathBuf| {
        let path_string = x.display().to_string().to_lowercase();
        if searchword.is_empty() {
            return true;
        }

        for wd in searchword.split_whitespace() {
            if !path_string.contains(&wd.to_lowercase()) {
                return false;
            }
        }

        true
    }
}

pub fn opendir(fpath: &Path) -> Result<(), std::io::Error> {
    // 対象ファイルのフォルダをファインダーで開く
    #[cfg(target_os = "macos")]
    let parentpath = fpath.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "parent path isnot found")
    })?;

    Command::new("open").arg(parentpath).spawn()?;

    #[cfg(target_os = "windows")]
    Command::new("explorer")
        .arg(fpath.parent().unwrap())
        .spawn()?;

    #[cfg(target_os = "linux")]
    Command::new("xdg-open")
        .arg(fpath.parent().unwrap())
        .spawn()?;

    Ok(())
}
