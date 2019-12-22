mod interface;
mod manager;

use crate::{
    interface::*,
    manager::{
        Manager,
        SpawnError,
    },
};
use futures::FutureExt;
use polldaddy::Quiz;
use std::time::Duration;
use tokio::runtime::Runtime;

fn proxy_spam(ctx: &mut AppContext) {
    let quiz = ctx.quiz.clone();
    let option = ctx.option;

    ctx.rt.block_on(async {
        let mut should_exit = false;
        let manager = Manager::new(quiz, option);

        while !should_exit {
            println!("Locating proxies...");
            let maybe_data = futures::select! {
                wait = manager.fetch_proxies_and_spawn(Duration::from_secs(10)).fuse() => {
                    Some(wait)
                },
                exit = tokio::signal::ctrl_c().fuse() => {
                    None  // Exit if the ctrl c handler fails to bind
                },
            };

            match maybe_data {
                Some(Some((proxy_info_list, spawn_results))) => {
                    println!("Found Proxies: ");
                    for (i, proxy) in proxy_info_list.iter().enumerate() {
                        println!("{})", i + 1,);
                        println!("Country: {}", proxy.country_name());
                        println!("Url: {}", proxy.get_url());
                        println!();
                    }

                    for res in spawn_results.iter() {
                        match res {
                            Ok(_) => (),
                            Err(SpawnError::DuplicateProxy) => {
                                println!("Failed to spawn worker with duplicate proxy");
                            }
                            Err(e) => {
                                println!("Failed to spawn worker, got error: {:#?}", e);
                            }
                        }
                    }
                }
                Some(None) => {
                    println!("Failed to get proxy list");
                }
                None => {
                    should_exit = true;
                }
            }

            println!();
            println!("# of Workers: {}", manager.len());

            while let Ok(msg) = manager.read_message() {
                match msg.data {
                    Ok(res) => {
                        match res.html() {
                            Ok(h) => {
                                println!("{}", HtmlResponseRefDisplay(h));
                                println!("Registered vote: {}", res.registered_vote());
                                println!();
                            }
                            Err(e) => {
                                dbg!(e);
                            }
                        }
                        if res.html().is_err() {
                            dbg!(res);
                        }
                    }
                    Err(_e) => {
                        println!("Invalid, exiting worker #{}", msg.id);
                        manager.shutdown_worker(msg.id);
                    }
                }
            }
        }
        println!("Exiting...");
        let zombies = manager.exit().await;
        println!("Workers Remaining: {}", zombies);
    });
}

fn local_spam(ctx: &mut AppContext) {
    let quiz = &ctx.quiz;
    let client = ctx.client.clone();
    let option = ctx.option;

    let mut vote_count: usize = 0;

    println!("Enter a delay in seconds (10 is the default): ");
    let delay: u64 = match read_parse() {
        Some(v) => v,
        None => {
            println!("Invalid input");
            return;
        }
    };

    println!();
    println!("Using delay of {} sec(s)", delay);
    println!();

    ctx.rt.block_on(async {
        let mut should_exit = false;
        while !should_exit {
            println!("Sending Vote #{}...", vote_count + 1);
            match client.vote(&quiz, option).await {
                Ok(res) => {
                    println!();
                    match res.html() {
                        Ok(html) => {
                            println!("{}", HtmlResponseRefDisplay(html));
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

pub struct AppContext {
    rt: Runtime,
    client: polldaddy::Client,
    quiz: Quiz,
    option: usize,
}

fn main() {
    let mut rt = match tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_time()
        .enable_io()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            println!("Failed to init Tokio Runtime, got: {:#?}", e);
            return;
        }
    };

    let client = polldaddy::Client::new();

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

    if quizzes.is_empty() {
        println!("No quizzes located.");
        return;
    }

    let maybe_quiz = quizzes.swap_remove(0);

    let quiz = match maybe_quiz {
        Ok(q) => q,
        Err(e) => {
            println!("Failed to parse quiz data. Got error: {:#?}", e);
            return;
        }
    };

    println!("Found quiz.");

    println!();
    println!("{}", QuizRefDisplay(&quiz));
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

    let mut ctx = AppContext {
        rt,
        client,
        quiz,
        option: answer_index - 1,
    };

    println!("Would you like to use the experimental proxy-spam feature (Y/N)?");
    match read_string().chars().next() {
        Some('y') | Some('Y') => proxy_spam(&mut ctx),
        _ => local_spam(&mut ctx),
    }
}
