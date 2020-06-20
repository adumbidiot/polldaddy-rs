use free_proxy_list::Client;
use std::time::Duration;

#[test]
fn it_works() {
    let client = Client::new();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let res = rt.block_on(client.get_list()).unwrap();
    assert!(!res.is_empty());
    assert!(res.iter().any(|el| el.is_ok()));

    let ret = rt.block_on(free_proxy_list::probe(
        res.iter().filter_map(|el| el.as_ref().ok()),
        Duration::from_secs(5),
    ));

    let good = res
        .iter()
        .filter_map(|el| el.as_ref().ok())
        .zip(ret.iter())
        .filter(|(_, good)| **good)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    dbg!(good);
}
