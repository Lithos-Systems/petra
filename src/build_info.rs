pub fn build_info_string() -> String {
    let info = crate::build_info();
    format!(
        "version: {}, commit: {} built {}",
        info.version,
        info.git_commit.unwrap_or("unknown"),
        info.build_timestamp
    )
}
