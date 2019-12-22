pub mod proxy_info;

pub use crate::proxy_info::{
    Anonymity,
    ProxyInfo,
    ProxyInfoError,
};
use bytes::buf::ext::BufExt;
pub use isocountry::CountryCode;
use select::{
    document::Document,
    predicate::{
        Attr,
        Name,
    },
};
use std::time::Duration;

pub type ProxyResult<T> = Result<T, ProxyError>;

#[derive(Debug)]
pub enum ProxyError {
    Reqwest(reqwest::Error),
    Io(std::io::Error),

    MissingTable,
}

impl From<reqwest::Error> for ProxyError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<std::io::Error> for ProxyError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Default)]
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Client {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_list(&self) -> ProxyResult<Vec<Result<ProxyInfo, ProxyInfoError>>> {
        let res = self
            .client
            .get("https://free-proxy-list.net/")
            .send()
            .await?;

        let body = res.bytes().await?;
        let doc = Document::from_read(body.reader())?;

        let table = doc
            .find(Attr("id", "proxylisttable"))
            .last()
            .ok_or(ProxyError::MissingTable)?;

        let table_body = table
            .find(Name("tbody"))
            .last()
            .ok_or(ProxyError::MissingTable)?
            .find(Name("tr"))
            .map(ProxyInfo::from_node)
            .collect::<Vec<_>>();

        Ok(table_body)
    }
}

pub async fn probe<'a, T: Iterator<Item = &'a ProxyInfo>>(iter: T, timeout: Duration) -> Vec<bool> {
    use futures::future::join_all;

    let iter = iter.map(|info| async move {
        let url = info.get_url();
        let proxy = match reqwest::Proxy::all(&url) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let client = match reqwest::Client::builder()
            .timeout(timeout)
            .proxy(proxy)
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        let res = match client.get("http://whatsmyip.me/").send().await {
            Ok(r) => r,
            Err(_) => {
                return false;
            }
        };

        if !res.status().is_success() {
            return false;
        }

        res.text().await.is_ok()
    });

    join_all(iter).await
}
