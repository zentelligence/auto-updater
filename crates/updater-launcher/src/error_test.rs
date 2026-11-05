use super::*;

#[test]
fn undeclared_command_message_names_the_command_and_version() {
    let error = LauncherError::UndeclaredCommand {
        name: "tool-worker".to_string(),
        version: "1.10.0".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "command \"tool-worker\" is not declared by the active release set (version 1.10.0)"
    );
}

#[test]
fn core_error_is_wrapped_transparently() {
    let core_error = CoreError::EmptyTargetTriple;
    let error = LauncherError::from(core_error);

    assert_eq!(error.to_string(), "target triple must not be empty");
}
