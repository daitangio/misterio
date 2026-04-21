use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HostConfig {
    pub(crate) docker_context: Option<String>,
    pub(crate) docker_host: Option<String>,
}

pub(crate) fn load_host_config(
    config_path: &Path,
    host_target: &str,
) -> Result<HostConfig, String> {
    if !config_path.exists() {
        return Ok(HostConfig::default());
    }

    let raw = fs::read_to_string(config_path)
        .map_err(|error| format!("failed to read {}: {error}", config_path.display()))?;
    Ok(parse_host_config(&raw, host_target))
}

pub(crate) fn parse_host_config(raw: &str, host_target: &str) -> HostConfig {
    let mut current_section = String::new();
    let mut current_subsection = String::new();
    let mut config = HostConfig::default();

    for line in raw.lines() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(section) = line
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
        {
            current_section.clear();
            current_subsection.clear();

            let mut parts = section.splitn(3, '.');
            current_section.push_str(parts.next().unwrap_or_default().trim());
            current_subsection.push_str(parts.next().unwrap_or_default().trim());
            continue;
        }

        if current_section != host_target || current_subsection != "docker" {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        match key.trim() {
            "context" => config.docker_context = Some(parse_toml_string(value.trim())),
            "host" => config.docker_host = Some(parse_toml_string(value.trim())),
            _ => {}
        }
    }

    config
}

fn strip_comment(line: &str) -> &str {
    let mut in_quotes = false;

    for (index, ch) in line.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '#' if !in_quotes => return &line[..index],
            _ => {}
        }
    }

    line
}

fn parse_toml_string(value: &str) -> String {
    if let Some(inner) = value.strip_prefix('"').and_then(|item| item.strip_suffix('"')) {
        inner.replace("\\\"", "\"")
    } else {
        value.to_string()
    }
}
