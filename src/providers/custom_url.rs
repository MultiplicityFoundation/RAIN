use anyhow::bail;

pub(crate) fn parse_custom_provider_url(
    raw_url: &str,
    provider_kind: &str,
    _example: &str,
) -> anyhow::Result<String> {
    let base_url = raw_url.trim();
    if base_url.is_empty() {
        bail!("{provider_kind} custom provider URL cannot be empty");
    }

    let parsed = reqwest::Url::parse(base_url).map_err(|_| {
        anyhow::anyhow!(
            "{provider_kind} custom provider URL must be a valid absolute URL: {base_url}"
        )
    })?;

    match parsed.scheme() {
        "http" | "https" => Ok(base_url.to_string()),
        scheme => bail!(
            "{provider_kind} custom provider URL must use http or https scheme, got '{scheme}'"
        ),
    }
}
