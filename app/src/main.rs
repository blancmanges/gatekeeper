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
use gatekeeper::bb_api::repositories::username::repo_slug::pullrequests::PullRequest;
use gatekeeper::get_commands;

use std::collections::HashMap;
use std::collections::hash_map::RandomState;

fn main() {
    let mut logger = sloggers::terminal::TerminalLoggerBuilder::new();
    logger.level(sloggers::types::Severity::Trace);
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
            Arg::with_name("repo_owner")
                .long("--bitbucket-repo-owner")
                .takes_value(true)
                .required(true)
                .env("BITBUCKET_REPO_OWNER")
                .hide_env_values(true),
        )
        .arg(
            Arg::with_name("repo_slugs")
                .long("--bitbucket-repo-slugs")
                .takes_value(true)
                .required(true)
                .env("BITBUCKET_REPO_SLUGS")
                .hide_env_values(true),
        )
        .get_matches();

    trace!(logger, "Retrieving bitbucket_username");
    let bitbucket_username = app_args.value_of("bitbucket_username").unwrap();
    trace!(logger, "Retrieving bitbucket_password");
    let bitbucket_password = app_args.value_of("bitbucket_password").unwrap();
    trace!(logger, "Retrieving repo_owner");
    let repo_owner = app_args.value_of("repo_owner").unwrap();
    trace!(logger, "Retrieving repo_slug");
    let repo_slugs = app_args.value_of("repo_slugs").unwrap();

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

    for repo_slug in repo_slugs.split(',') {
        println!("{}", repo_slug);
        println!("------------------------------------------------------------------------");

        debug!(logger, "Building URLs");
        let repo_base_url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}",
            repo_owner, repo_slug
        );
        let repo_url_prs = format!("{}/pullrequests/", repo_base_url);

        debug!(logger, "Obtaining first page of PR list");
        let mut prs_first = reqwest_get(repo_url_prs.as_str()).send().unwrap();
        debug!(logger, "Obtaining all pages or PR list");
        trace!(logger, "Getting PRs list first page text");
        let prs_first_txt = prs_first.text().unwrap();
        trace!(logger, "Response: {}", prs_first_txt);
        trace!(logger, "Deserializing PRs list first page text");
        let prs_first: Paginated<PullRequest> =
            serde_json::from_str(prs_first_txt.as_str()).unwrap();
        trace!(logger, "Getting remaining pages of PRs list");
        let prs = unpaginate(prs_first, &reqwest_get, &logger).unwrap();

        debug!(logger, "PRs: {:?}", prs);

        for pr in &prs {
            debug!(logger, "PR id: {} title: {}", pr.id, pr.title);

            debug!(logger, "PR: {:?}", pr);

            let mut pr_first = reqwest_get(pr.links.activity.href.as_str()).send().unwrap();
            let pr_first_txt = pr_first.text().unwrap();
            trace!(logger, "PR first text: {}", pr_first_txt);
            let pr_first: Paginated<ActivityItem> =
                serde_json::from_str(pr_first_txt.as_str()).unwrap();
            let mut pr_activity = unpaginate(pr_first, &reqwest_get, &logger).unwrap();
            pr_activity.reverse();
            debug!(logger, "PR: {:?}", pr_activity);
            let cmds = get_commands(pr_activity);
            debug!(logger, "Commands: {:?}", cmds);

            let mut review_status: HashMap<String, String, RandomState> = HashMap::new();
            for user_cmd in cmds {
                match user_cmd.command.as_str() {
                    "\\+1" | "\\+0" | "-1" => {
                        let vote = review_status
                            .entry(user_cmd.user.to_string())
                            .or_insert(String::from("0"));
                        *vote = user_cmd.command;
                    }
                    _ => {}
                }
            }

            println!("  PR {}: {}", pr.id, pr.title);
            println!("    -- author: {}", pr.author.username);
            println!(
                "    -- link: https://bitbucket.org/{}/{}/pull-requests/{}",
                repo_owner, repo_slug, pr.id
            );
            for (user, vote) in &review_status {
                println!("    {}: {}", user, vote);
            }
        }
    }
}
