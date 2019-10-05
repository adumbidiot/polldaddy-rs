use polldaddy::Client;
use std::{
    io::{
        stdin,
        stdout,
        Write,
    },
    str::FromStr,
    time::Duration,
};

fn read_string() -> String {
    let mut s = String::new();
    let _ = stdout().flush();
    stdin().read_line(&mut s).unwrap();
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}

fn read_parse<T: FromStr>() -> Option<T> {
    read_string().parse().ok()
}

fn main() {
    let delay = 1000 * 10;
    let mut i = 0;
    let client = Client::new();

    println!("Enter the target url: ");
    let url = read_string();

    println!("Using delay of {} ms", delay);

    println!("Scanning Url: {}", url);
    let mut quizzes = match client.quiz_from_url(&url) {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to scan url, got error: {:#?}", e);
            return;
        }
    };

    let maybe_quiz = match quizzes.pop() {
        Some(q) => q,
        None => {
            println!("No quizzes located.");
            return;
        }
    };

    println!("Found quiz.");

    let quiz = match maybe_quiz {
        Ok(q) => q,
        Err(e) => {
            println!("Failed to parse quiz data. Got error: {:#?}", e);
            return;
        }
    };

    println!();
    println!("Quiz Id: {}", quiz.get_id());
    println!("Quiz Hash: {}", quiz.get_hash());
    println!("Quiz Closed: {}", quiz.is_closed());
    println!("Quiz Referer: {}", quiz.get_referer());
    println!("Quiz Va: {}", quiz.get_va());
    println!("Answers: ");
    for (i, ans) in quiz.get_answers().iter().enumerate() {
        println!("{}) {} | Code {}", i + 1, ans.get_text(), ans.get_id());
    }
    println!();

    if quiz.is_closed() {
        println!("Quiz Closed.");
        return;
    }

    let answers_len = quiz.get_answers().len();

    println!("Enter an answer index (1 - {}): ", answers_len);
    let answer_index: usize = match read_parse() {
        Some(v) => v,
        None => {
            println!("Invalid input");
            return;
        }
    };

    if answer_index < 1 || answer_index > answers_len {
        println!("Invalid answer index");
        return;
    }

    println!();

    loop {
        println!("Sending Vote #{}...", i + 1);
        match client.vote(&quiz, answer_index - 1) {
            Ok(res) => {
                println!("Response Data: {}", res.data);
                println!();
                if let Some(html) = res.html_response.as_ref() {
                    for (i, res) in html.get_answers().iter().enumerate() {
                        if let Some(res) = res {
                            println!(
                                "{}) {} | {} votes | {}%",
                                i + 1,
                                res.get_text(),
                                res.get_votes(),
                                res.get_percent()
                            );
                        } else {
                            println!("{}) Failed to parse", i + 1);
                        }
                    }

                    println!("Total Votes: {} votes", html.get_total_votes());
                }

                if res.is_banned() {
                    println!("Error: You have been IP Banned. Try using another IP Address.");
                }
            }
            Err(e) => {
                println!("Failed to submit vote, got error: {:#?}", e);
            }
        }
        i += 1;

        println!();

        std::thread::sleep(Duration::from_millis(delay));
    }
}
