// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use(crate_name, crate_version, crate_description)]
extern crate clap;
extern crate gatekeeper;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate sloggers;

use clap::{App, Arg};
use sloggers::Build;
use gatekeeper::unpaginate;
use gatekeeper::bb_api::Paginated;
use gatekeeper::bb_api::repositories::username::repo_slug::pullrequests::
    pull_request_id::activity::ActivityItem;

fn main() {
    let mut logger = sloggers::terminal::TerminalLoggerBuilder::new();
    logger.level(sloggers::types::Severity::Debug);
    logger.destination(sloggers::terminal::Destination::Stderr);
    let logger = logger.build().unwrap();

    debug!(logger, "Initializing & parsing cmdline args");
    let app_args = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("bitbucket_username")
                .long("--bitbucket-username")
                .short("-u")
                .takes_value(true)
                .required(true)
                .env("BITBUCKET_USERNAME")
                .hide_env_values(true),
        )
        .arg(
            Arg::with_name("bitbucket_password")
                .long("--bitbucket-password")
                .short("-p")
                .takes_value(true)
                .required(true)
                .env("BITBUCKET_PASSWORD")
                .hide_env_values(true),
        )
        .arg(
            Arg::with_name("bitbucket_pr_url")
                .long("--bitbucket-repo-rul")
                .takes_value(true)
                .required(true)
                .env("BITBUCKET_REPO_URL")
                .hide_env_values(true),
        )
        .get_matches();

    debug!(logger, "Retrieving bitbucket_username");
    let bitbucket_username = app_args.value_of("bitbucket_username").unwrap();
    debug!(logger, "Retrieving bitbucket_password");
    let bitbucket_password = app_args.value_of("bitbucket_password").unwrap();
    debug!(logger, "Retrieving bitbucket_pr_url");
    let bitbucket_pr_url = app_args.value_of("bitbucket_pr_url").unwrap();

    debug!(logger, "Creating reqwest::Client");
    let client = reqwest::Client::new();

    debug!(logger, "Creating Request");
    let reqwest_get = |url: &str| {
        let mut req_builder = client.get(url);
        req_builder
            .basic_auth(bitbucket_username, Some(bitbucket_password))
            .header(reqwest::header::ContentType::json());
        req_builder
    };
    if let Ok(mut res) = reqwest_get(bitbucket_pr_url).send() {
        debug!(logger, "Obtaining request response text");
        if let Ok(res_text) = res.text() {
            trace!(logger, "Res: {}", res_text);
            let act: Result<Paginated<ActivityItem>, serde_json::Error> =
                serde_json::from_str(res_text.as_str());
            if let Ok(activity) = act {
                debug!(logger, "{:?}", activity);
                let all_res = unpaginate(activity, reqwest_get, &logger).unwrap();
                debug!(logger, "{:?}", all_res);
            }
        }
    }
}
