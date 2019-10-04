use polldaddy::Client;
use std::{
    io::{
        stdin,
        stdout,
        Write,
    },
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
    let quiz = quizzes.pop().unwrap().unwrap();
    println!("Using Quiz: {:#?}", quiz);
    let answer_index = quiz
        .get_answers()
        .iter()
        .position(|el| el.get_text().to_lowercase().contains("rocklin"))
        .unwrap();
    println!();

    loop {
        println!("Sending Vote #{}...", i + 1);
        let res = client.vote(&quiz, answer_index).unwrap();
        i += 1;
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
        println!();

        std::thread::sleep(Duration::from_millis(delay));
    }
}
