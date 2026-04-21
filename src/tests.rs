use crate::app::{
    determine_fixed_port, determine_instance_name, role_matches, role_name_from_env,
};
use crate::config::parse_host_config;

#[test]
fn instance_names_match_previous_rules() {
    assert_eq!(determine_instance_name("pgvector"), "pgvector");
    assert_eq!(determine_instance_name("pgvector@1"), "pgvector_1");
}

#[test]
fn fixed_port_stays_stable_for_instances() {
    assert_eq!(determine_fixed_port("pgvector", 7000), 7008);
    assert_eq!(determine_fixed_port("pgvector@1", 7000), 7011);
}

#[test]
fn role_name_ignores_instance_suffix() {
    assert_eq!(role_name_from_env("pgvector.env"), "pgvector");
    assert_eq!(role_name_from_env("pgvector@2.env"), "pgvector");
}

#[test]
fn role_matching_is_exact_for_role_or_instance() {
    assert!(role_matches("pgvector.env", "pgvector"));
    assert!(role_matches("pgvector@2.env", "pgvector"));
    assert!(role_matches("pgvector@2.env", "pgvector@2"));
    assert!(!role_matches("pgvector.env", "pg"));
    assert!(!role_matches("pgvector@2.env", "vector"));
}

#[test]
fn minimal_host_config_parser_reads_docker_block() {
    let raw = r#"
    [alice.docker]
    context = "orbstack"
    host = "ssh://alice"

    [bob.docker]
    host = "ssh://bob"
    "#;

    let config = parse_host_config(raw, "alice");
    assert_eq!(config.docker_context.as_deref(), Some("orbstack"));
    assert_eq!(config.docker_host.as_deref(), Some("ssh://alice"));
}
