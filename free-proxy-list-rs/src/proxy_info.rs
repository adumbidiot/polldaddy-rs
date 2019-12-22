use isocountry::CountryCode;
use select::{
    node::Node,
    predicate::Text,
};
use std::{
    net::IpAddr,
    str::FromStr,
};

#[derive(Debug)]
pub enum ProxyInfoError {
    MissingIp,
    InvalidIp(std::net::AddrParseError),

    MissingPort,
    InvalidPort(std::num::ParseIntError),

    MissingCountryName,

    MissingCountry,
    InvalidCountry(isocountry::CountryCodeParseErr),

    MissingAnonymity,
    InvalidAnonymity(AnonymityParseError),

    MissingGoogle,
    InvalidGoogle(String),

    MissingHttps,
    InvalidHttps(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProxyInfo {
    ip: IpAddr,
    port: u32,
    country_code: CountryCode,
    country_name: String,
    anonymity: Anonymity,
    google: bool,
    https: bool,
}

impl ProxyInfo {
    pub fn from_node(node: Node) -> Result<Self, ProxyInfoError> {
        let mut iter = node
            .children()
            .filter(|el| el.name() == Some("td"))
            .map(|el| el.find(Text).last()?.as_text());

        let ip = iter
            .next()
            .flatten()
            .ok_or(ProxyInfoError::MissingIp)?
            .parse()
            .map_err(ProxyInfoError::InvalidIp)?;

        let port = iter
            .next()
            .flatten()
            .ok_or(ProxyInfoError::MissingPort)?
            .parse()
            .map_err(ProxyInfoError::InvalidPort)?;

        let country_code = iter
            .next()
            .flatten()
            .map(CountryCode::for_alpha2)
            .ok_or(ProxyInfoError::MissingCountry)?
            .map_err(ProxyInfoError::InvalidCountry)?;

        let country_name = iter
            .next()
            .flatten()
            .ok_or(ProxyInfoError::MissingCountryName)?
            .to_string();

        let anonymity = iter
            .next()
            .flatten()
            .ok_or(ProxyInfoError::MissingAnonymity)?
            .parse()
            .map_err(ProxyInfoError::InvalidAnonymity)?;

        let google = iter
            .next()
            .flatten()
            .ok_or(ProxyInfoError::MissingGoogle)
            .and_then(|s| match s {
                "yes" => Ok(true),
                "no" => Ok(false),
                s => Err(ProxyInfoError::InvalidGoogle(String::from(s))),
            })?;

        let https = iter
            .next()
            .flatten()
            .ok_or(ProxyInfoError::MissingHttps)
            .and_then(|s| match s {
                "yes" => Ok(true),
                "no" => Ok(false),
                s => Err(ProxyInfoError::InvalidHttps(String::from(s))),
            })?;

        Ok(ProxyInfo {
            ip,
            port,
            country_code,
            country_name,
            anonymity,
            google,
            https,
        })
    }

    pub fn ip(&self) -> &IpAddr {
        &self.ip
    }

    pub fn port(&self) -> u32 {
        self.port
    }

    pub fn country_name(&self) -> &str {
        &self.country_name
    }

    pub fn anonymity(&self) -> Anonymity {
        self.anonymity
    }

    pub fn get_url(&self) -> String {
        let protocol = if self.https { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.ip, self.port)
    }
}

#[derive(Debug)]
pub struct AnonymityParseError(String);

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Anonymity {
    Transparent,
    EliteProxy,
    Anonymous,
}

impl FromStr for Anonymity {
    type Err = AnonymityParseError;

    fn from_str(s: &str) -> Result<Self, AnonymityParseError> {
        match s {
            "transparent" => Ok(Anonymity::Transparent),
            "elite proxy" => Ok(Anonymity::EliteProxy),
            "anonymous" => Ok(Anonymity::Anonymous),
            s => Err(AnonymityParseError(String::from(s))),
        }
    }
}
