use polldaddy::Quiz;
use std::{
    io::{
        stdin,
        stdout,
        Write,
    },
    str::FromStr,
};

pub fn read_string() -> String {
    let mut s = String::new();
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut s);
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}

pub fn read_parse<T: FromStr>() -> Option<T> {
    read_string().parse().ok()
}

pub struct HtmlResponseRefDisplay<'a>(pub &'a polldaddy::HtmlResponse);

impl<'a> std::fmt::Display for HtmlResponseRefDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, res) in self.0.get_answers().iter().enumerate() {
            match res {
                Ok(res) => {
                    writeln!(
                        f,
                        "{}) {} | {} votes | {}%",
                        i + 1,
                        res.get_text(),
                        res.get_votes(),
                        res.get_percent()
                    )?;
                }
                Err(e) => {
                    writeln!(f, "{}) Failed to parse, got error: {:?}", i + 1, e)?;
                }
            };
        }

        writeln!(f, "\nTotal Votes: {} votes", self.0.get_total_votes())?;
        Ok(())
    }
}

pub struct QuizRefDisplay<'a>(pub &'a Quiz);

impl<'a> std::fmt::Display for QuizRefDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Quiz Id: {}", self.0.get_id())?;
        writeln!(f, "Quiz Hash: {}", self.0.get_hash())?;
        writeln!(f, "Quiz Closed: {}", self.0.is_closed())?;
        writeln!(f, "Quiz Referer: {}", self.0.get_referer())?;
        writeln!(f, "Quiz Va: {}", self.0.get_va())?;
        writeln!(f)?;
        writeln!(f, "Answers: ")?;
        for (i, ans) in self.0.get_answers().iter().enumerate() {
            writeln!(f, "{}) {} | Code {}", i + 1, ans.get_text(), ans.get_id())?;
        }
        Ok(())
    }
}
