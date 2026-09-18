//! Exact JavaScript toolchain pins shared by setup and diagnostics.

const NODE_PIN: &str = include_str!("../.node-version");
const NPM_PIN: &str = include_str!("../.npm-version");

fn pin(text: &'static str) -> &'static str {
    let value = text
        .strip_suffix('\n')
        .filter(|value| !value.contains(['\n', '\r']))
        .expect("toolchain pin must be one LF-terminated line");
    let mut parts = value.split('.');
    assert!(
        parts.clone().count() == 3
            && parts
                .all(|part| { !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()) }),
        "toolchain pin must be an exact three-component numeric version"
    );
    value
}

pub(crate) fn node() -> &'static str {
    pin(NODE_PIN)
}

pub(crate) fn npm() -> &'static str {
    pin(NPM_PIN)
}

pub(crate) fn matches_node(output: &str) -> bool {
    output.strip_prefix('v') == Some(node())
}

pub(crate) fn matches_npm(output: &str) -> bool {
    output == npm()
}

#[cfg(test)]
mod tests {
    #[test]
    fn pins_are_exact_and_newer_is_not_implicitly_admitted() {
        assert_eq!(super::node(), "22.23.2");
        assert_eq!(super::npm(), "10.9.8");
        assert!(super::matches_node("v22.23.2"));
        assert!(!super::matches_node("v22.23.1"));
        assert!(!super::matches_node("v26.1.0"));
        assert!(super::matches_npm("10.9.8"));
        assert!(!super::matches_npm("11.13.0"));
    }
}
