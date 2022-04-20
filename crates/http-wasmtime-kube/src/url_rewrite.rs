use anyhow::{anyhow, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

const KUBERNETES_INTERNAL_SVC_FQDN: &str = "kubernetes.default.svc";

pub(crate) fn url_rewrite_workaround(
    server_url: &http::uri::Uri,
) -> Result<(http::uri::Uri, Option<SocketAddr>)> {
    let host = server_url.host().expect("request doesn't have an host");
    let port = server_url
        .port_u16()
        .or_else(|| {
            match server_url
                .scheme()
                .unwrap_or_else(|| &http::uri::Scheme::HTTP)
                .as_str()
            {
                "http" => Some(80),
                "https" => Some(443),
                _ => None,
            }
        })
        .ok_or(anyhow!("url doesn't use a known schema"))?;

    let socket_addr = if let Ok(ipv4_addr) = host.parse::<Ipv4Addr>() {
        Some(SocketAddr::new(IpAddr::V4(ipv4_addr), port))
    } else if let Ok(ipv6_addr) = host.parse::<Ipv6Addr>() {
        Some(SocketAddr::new(IpAddr::V6(ipv6_addr), port))
    } else {
        None
    };

    if socket_addr.is_some() {
        let authority = server_url
            .authority()
            .unwrap()
            .as_str()
            .replace(host, KUBERNETES_INTERNAL_SVC_FQDN);

        let uri = http::uri::Builder::new()
            .scheme(server_url.scheme_str().unwrap())
            .authority(authority.as_str())
            .path_and_query(server_url.path_and_query().unwrap().to_string())
            .build()?;
        Ok((uri, socket_addr))
    } else {
        Ok((server_url.clone(), None))
    }
}
