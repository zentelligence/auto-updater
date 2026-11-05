use super::*;

#[test]
fn config_errors_exit_with_code_two() {
    let error = CliError::MissingConfig {
        variable: "UPDATER_PRODUCT".to_string(),
    };

    assert_eq!(error.exit_code(), 2);
}

#[test]
fn engine_errors_exit_with_code_one() {
    let error = CliError::Engine(updater_engine::EngineError::NoPreviousRelease);

    assert_eq!(error.exit_code(), 1);
}

#[test]
fn missing_config_message_names_the_variable() {
    let error = CliError::MissingConfig {
        variable: "UPDATER_PRODUCT".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "missing required environment variable UPDATER_PRODUCT"
    );
}
