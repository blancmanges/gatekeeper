// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate failure;
extern crate gatekeeper;
extern crate regex;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate sloggers;
#[macro_use]
extern crate structopt;

use gatekeeper::bitbucket::BitBucketApiBasicAuth;
use gatekeeper::bitbucket::PullRequest;
use gatekeeper::bitbucket::values_from_all_pages;
use gatekeeper::bitbucket::ActivityItem;
use gatekeeper::PullRequestState;
use gatekeeper::RepositoryURLs;

use failure::Error;
use sloggers::Build;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    #[structopt(short = "u", long = "bitbucket-username")]
    bitbucket_username: String,
    #[structopt(short = "p", long = "bitbucket-password")]
    bitbucket_password: String,
    #[structopt(short = "o", long = "bitbucket-repo-owner")]
    repo_owner: String,
    #[structopt(short = "r", long = "bitbucket-repo-slug")]
    repo_slugs: Vec<String>,
}

fn main() {
    let logger = {
        let mut logger = sloggers::terminal::TerminalLoggerBuilder::new();
        logger.level(sloggers::types::Severity::Trace);
        logger.destination(sloggers::terminal::Destination::Stderr);
        logger.build().unwrap()
    };

    app(&logger).unwrap();
}

fn app(logger: &slog::Logger) -> Result<(), Error> {
    debug!(logger, "Starting application");
    trace!(logger, "Processing args");
    let app_args = Opt::from_args();

    trace!(logger, "Setting up BitBucket basic auth");
    let client = BitBucketApiBasicAuth::new(
        app_args.bitbucket_username,
        app_args.bitbucket_password,
        reqwest::Client::new(),
    );

    debug!(logger, "Repositories to process: {:?}", app_args.repo_slugs);
    for repo_slug in &app_args.repo_slugs {
        repo_prs(&app_args.repo_owner, &repo_slug, &client, &logger)?;
    }

    Ok(())
}

fn repo_prs(
    repo_owner: &str,
    repo_slug: &str,
    client: &BitBucketApiBasicAuth,
    logger: &slog::Logger,
) -> Result<(), Error> {
    let logger = logger.new(o!(
        "repo_owner" => repo_owner.to_string(),
        "repo_slug" => repo_slug.to_string(),
    ));

    debug!(logger, "Processing repo");

    println!("{}", repo_slug);
    println!("------------------------------------------------------------------------");

    let urls = RepositoryURLs::new(repo_owner, repo_slug);

    debug!(logger, "Obtaining BB/{{repo}}/pullrequests/");
    let pullrequests = values_from_all_pages(&urls.api_url, &client, &logger)?;
    trace!(logger, "PRs: {:?}", pullrequests);

    for pr in pullrequests {
        repo_pr(pr, &urls, &client, &logger)?;
    }

    Ok(())
}

fn repo_pr(
    pr: PullRequest,
    urls: &RepositoryURLs,
    client: &BitBucketApiBasicAuth,
    logger: &slog::Logger,
) -> Result<(), Error> {
    let logger = logger.new(o!(
        "pr_id" => pr.id,
    ));

    let urls = urls.with_id(pr.id);

    debug!(logger, "PR title: {}", pr.title);
    trace!(logger, "PR: {:?}", pr);
    trace!(logger, "Urls: {:?}", urls);

    debug!(logger, "Obtaining BB/{{repo}}/pullrequests/{{id}}");
    let activity = {
        let mut activity =
            values_from_all_pages::<ActivityItem>(&pr.links.activity.href, &client, &logger)?;
        activity.reverse();
        activity
    };
    trace!(logger, "Activity: {:?}", activity);
    let pr_state = PullRequestState::from_activity(activity, &logger)?;

    println!("  PR {}: {}", pr.id, pr.title);
    println!("    -- author: {}", pr.author.username);
    println!("    -- link: {}", urls.web_url);
    for (user, status) in &pr_state.review_status {
        println!("    {}: {:?}", user, status);
    }

    Ok(())
}
