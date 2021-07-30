#[derive(Debug)]
pub struct OstreeOpts {
    pub hostname: String,
    pub ostree_name_remote: String,
    pub ostree_gpg_verify: bool,
    pub ostreepush_ssh_port: String,
    pub ostreepush_ssh_user: String,
    pub ostreepush_ssh_pwd: String,
}
