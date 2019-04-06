// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use gatekeeper::{
    bitbucket::{values_from_all_pages, ActivityItem, BitBucketApiBasicAuth, PullRequest},
    PullRequestState, RepositoryURLs,
};

use failure::Error;
use itertools::Itertools;
use slog::{debug, error, o, trace, Drain, FnValue};
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Error>;

#[derive(structopt::StructOpt, Debug)]
#[structopt()]
struct Opt {
    #[structopt(short = "u", long = "bitbucket-username", env = "BITBUCKET_USERNAME")]
    bitbucket_username: String,
    #[structopt(short = "p", long = "bitbucket-password", env = "BITBUCKET_PASSWORD")]
    bitbucket_password: String,
    #[structopt(short = "o", long = "bitbucket-repo-owner", env = "REPO_OWNER")]
    repo_owner: String,
    #[structopt(
        short = "r",
        long = "bitbucket-repo-slug",
        env = "REPO_SLUGS",
        use_delimiter = true
    )]
    repo_slugs: Vec<String>,
}

fn main() {
    let logger = {
        let json_log_path = "gatekeeper.json.log";
        let json_log_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(json_log_path)
            .unwrap();
        let drain = slog_bunyan::new(json_log_file).build().fuse();
        let drain = slog_async::Async::new(drain).chan_size(256).build().fuse();
        slog::Logger::root(
            drain,
            o!(
                "src_code_module" => FnValue(|r| r.module()),
                "src_cloc_file" => FnValue(|r| r.file()),
                "src_cloc_line" => FnValue(|r| r.line()),
                "src_cloc_column" => FnValue(|r| r.column()),
            ),
        )
    };

    app(&logger).unwrap();
}

fn app(logger: &slog::Logger) -> Result<()> {
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
        let repo_prs = repo_prs(&app_args.repo_owner, &repo_slug, &client, &logger)?;

        trace!(logger, "Showing results for {}", repo_slug);
        display_repo(&repo_slug, &logger);
        for pr in repo_prs {
            display_pr_results(pr, &logger);
        }
    }

    debug!(logger, "Application execution completed");
    Ok(())
}

fn display_repo(repo_slug: &str, _logger: &slog::Logger) {
    println!("{}", repo_slug);
    println!("------------------------------------------------------------------------");
}

enum PullRequestProcessing {
    Success(PullRequestState),
    Failure(PullRequest, Error),
}

fn display_pr_results(res: PullRequestProcessing, logger: &slog::Logger) {
    match res {
        PullRequestProcessing::Failure(pr, e) => {
            error!(logger, "Error processing PR {}. Err: {}", pr.id, e);
            println!("  PR {}: {}", pr.id, pr.title);
            println!("    -- author: {}", pr.author.username);
            println!("    -- link: {}", pr.links.slf.href);
            println!("    PROCESSING ERROR");
        }
        PullRequestProcessing::Success(pr_state) => {
            println!("  PR {}: {}", pr_state.pr.id, pr_state.pr.title);
            println!("    -- author: {}", pr_state.pr.author.username);
            println!("    -- link: {}", pr_state.urls.web_url);
            println!("    -- current_hash: {:?}", pr_state.current_hash);
            if !pr_state.labels.is_empty() {
                println!("    -- labels: {}", pr_state.labels.iter().join(", "));
            }
            for (user, status) in &pr_state.review_status {
                println!("    {}: {:?}", user, status);
            }
        }
    }
}

fn repo_prs(
    repo_owner: &str,
    repo_slug: &str,
    client: &BitBucketApiBasicAuth,
    logger: &slog::Logger,
) -> Result<Vec<PullRequestProcessing>> {
    let logger = logger.new(o!(
        "repo_owner" => repo_owner.to_string(),
        "repo_slug" => repo_slug.to_string(),
    ));

    debug!(logger, "Processing repo");
    let urls = RepositoryURLs::new(repo_owner, repo_slug);

    trace!(logger, "Obtaining BB/{{repo}}/pullrequests/");
    let pullrequests = values_from_all_pages::<PullRequest>(&urls.api_url, &client, &logger)?;

    debug!(logger, "Pull requests: {:?}", pullrequests);
    let res = pullrequests
        .into_iter()
        .map(|pr| {
            repo_pr(pr.clone(), &urls, &client, &logger)
                .unwrap_or_else(|e| PullRequestProcessing::Failure(pr, e))
        })
        .collect();
    Ok(res)
}

fn repo_pr(
    pr: PullRequest,
    urls: &RepositoryURLs,
    client: &BitBucketApiBasicAuth,
    logger: &slog::Logger,
) -> Result<PullRequestProcessing> {
    let logger = logger.new(o!(
        "pr_id" => pr.id,
    ));
    debug!(logger, "Processing PR #{}: {}", pr.id, pr.title);
    trace!(logger, "PR: {:?}", pr);

    let urls = urls.with_id(pr.id);
    trace!(logger, "Urls: {:?}", urls);

    debug!(logger, "Obtaining PR activity");
    let activity = {
        let mut activity =
            values_from_all_pages::<ActivityItem>(&pr.links.activity.href, &client, &logger)?;
        activity.reverse();
        activity
    };
    trace!(logger, "Activity: {:?}", activity);

    let res = PullRequestState::from_activity(pr, activity, urls, &logger)?;
    Ok(PullRequestProcessing::Success(res))
}
