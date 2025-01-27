use crate::{
    model::{CrtShEntry, Subdomain},
    Error,
};
use reqwest::blocking::Client;
use std::{collections::HashSet, time::Duration};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};

pub fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    let entries: Vec<CrtShEntry> = http_client
        .get(format!("https://crt.sh/?q=%25.{}&output=json", target))
        .send()?
        .json()?;

    // clean and dedup results
    let mut subdomains: HashSet<&str> = entries
        .iter()
        .flat_map(|entry| {
            entry
                .name_value
                .split("\n")
                .map(|subdomain| subdomain.trim())
        })
        .filter(|subdomain| *subdomain != target)
        .filter(|subdomain| !subdomain.contains("*"))
        .collect();
    subdomains.insert(target);

    let subdomains: Vec<Subdomain> = subdomains
        .into_iter()
        .map(|domain| Subdomain {
            domain: domain.to_string(),
            open_ports: Vec::new(),
        })
        .filter(resolves)
        .collect();

    Ok(subdomains)
}

pub fn resolves(domain: &Subdomain) -> bool {
    let dns_resolver = Resolver::new(
        ResolverConfig::default(),
        ResolverOpts {
            timeout: Duration::from_secs(4),
            ..Default::default()
        },
    )
    .expect("subdomain resolver: building DNS client");
    dns_resolver.lookup_ip(domain.domain.as_str()).is_ok()
}
