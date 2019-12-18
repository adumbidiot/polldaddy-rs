use futures::FutureExt;
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
    let mut vote_count: usize = 0;
    let client = Client::new();

    let mut rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            println!("Failed to init Tokio Runtime, got: {:#?}", e);
            return;
        }
    };

    println!("Enter the target url: ");
    let url = read_string();
    println!();

    println!("Scanning Url: {}", url);
    let mut quizzes = match rt.block_on(client.quiz_from_url(&url)) {
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
    println!();
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

    println!("Enter a delay in seconds (10 is the default): ");
    let delay: u64 = match read_parse() {
        Some(v) => v,
        None => {
            println!("Invalid input");
            return;
        }
    };
    println!("Using delay of {} sec(s)", delay);

    println!();

    rt.block_on(async {
        let mut should_exit = false;
        while !should_exit {
            println!("Sending Vote #{}...", vote_count + 1);
            match client.vote(&quiz, answer_index - 1).await {
                Ok(res) => {
                    println!();
                    match res.html() {
                        Ok(html) => {
                            for (i, res) in html.get_answers().iter().enumerate() {
                                match res {
                                    Ok(res) => {
                                        println!(
                                            "{}) {} | {} votes | {}%",
                                            i + 1,
                                            res.get_text(),
                                            res.get_votes(),
                                            res.get_percent()
                                        );
                                    }
                                    Err(e) => {
                                        println!("{}) Failed to parse, got error: {:?}", i + 1, e);
                                    }
                                };
                            }

                            println!("Total Votes: {} votes", html.get_total_votes());
                        }
                        Err(e) => {
                            println!("Failed to parse html response, got error: {:#?}", e);
                        }
                    }

                    if res.is_ip_banned() {
                        println!("Error: You have been IP Banned. Try using another IP Address.");
                    } else if !res.registered_vote() {
                        println!("Error: Vote not registered. Cause Unknown.");
                    }
                }
                Err(e) => {
                    println!("Failed to submit vote, got error: {:#?}", e);
                }
            }

            vote_count += 1;
            println!();

            should_exit = futures::select! {
                wait = tokio::time::delay_for(Duration::from_secs(delay)).fuse() => Ok(false),
                exit = tokio::signal::ctrl_c().fuse() => exit.map(|_| true),
            }
            .unwrap_or(true); // Exit if the ctrl c handler fails to bind
        }
    });
}
