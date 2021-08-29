use crate::container_ostree::get_repo;
use crate::container_ostree::PATH_REPO_APPS;
use crate::ostree::OstreeOpts;
use ostree::gio;
use tracing::info;

pub fn _init_ostree_remotes(options: &OstreeOpts) -> Result<(), ()> {
    //// res = True
    let repo_container = get_repo(PATH_REPO_APPS);

    // self.ostree_remote_attributes = ostree_remote_attributes
    // opts = GLib.Variant('a{sv}', {'gpg-verify': GLib.Variant('b', ostree_remote_attributes['gpg-verify'])})
    // try:
    //     self.logger.info("Initalize remotes for the OS ostree: {}".format(ostree_remote_attributes['name']))
    //     if not ostree_remote_attributes['name'] in self.repo_os.remote_list():
    //         self.repo_os.remote_add(ostree_remote_attributes['name'],
    //                                 ostree_remote_attributes['url'],
    //                                 opts, None)
    //     self.remote_name_os = ostree_remote_attributes['name']

    let refs = repo_container
        .list_refs(None, gio::NONE_CANCELLABLE)
        .unwrap();
    info!(
        "Initalize remotes for the containers ostree: {:#?}",
        refs.keys()
    );
    let remote_list = repo_container.remote_list();
    let remote_list: Vec<&str> = remote_list.iter().map(|i| i.as_str()).collect();
    info!("remote_list {:#?}", remote_list);
    for remote in remote_list.iter() {
        let url = repo_container.remote_get_url(remote).unwrap();
        let url = url.as_str();
        info!("remote: {:#?} url: {:#?}", remote, url);
        if options.hostname == url {
            info!("reusing remote: {:#?} url: {:#?}", remote, url);
            // TODO ^ Try deleting the & and matching just "Ferris"
        } else {
            info!(
                "For remote {}, {} was expected and {} was received",
                remote, options.hostname, url
            );
        }
    }

    //         if remote_name not in self.repo_containers.remote_list():
    //             self.logger.info("We had the remote: {}".format(remote_name))
    //             self.repo_containers.remote_add(remote_name,
    //                                             ostree_remote_attributes['url'],
    //                                             opts, None)

    Ok(())
}
