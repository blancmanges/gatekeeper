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

use std::mem;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use gatekeeper::bitbucket::BitBucketApiBasicAuth;
use gatekeeper::bitbucket::PullRequest;
use gatekeeper::bitbucket::values_from_all_pages;
use gatekeeper::bitbucket::ActivityItem;
use gatekeeper::bitbucket::Approval;
use gatekeeper::RepositoryURLs;
use gatekeeper::ReviewStatus;

use failure::Error;
use regex::Regex;
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

fn app(logger: &slog::Logger) -> Result<(), Error> {
    debug!(logger, "Starting application");
    let app_args = Opt::from_args();
    let client = BitBucketApiBasicAuth::new(
        app_args.bitbucket_username,
        app_args.bitbucket_password,
        reqwest::Client::new(),
    );

    debug!(logger, "Repositories: {:?}", app_args.repo_slugs);
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

    let mut review_status: HashMap<String, ReviewStatus, RandomState> = HashMap::new();
    let re_vote = Regex::new(r"^(\\?\+|-)?\d$")?;

    for change in activity {
        trace!(logger, "Change: {:?}", change);
        match change {
            ActivityItem::Approval {
                approval: Approval { user },
            } => {
                let approve_user = user.username.to_string();
                debug!(logger, "User {:?} approves", approve_user);
                review_status.insert(approve_user, ReviewStatus::Voted { vote: 1 });
            }

            ActivityItem::Comment { comment } => {
                let comment_user = comment.user.username;

                for (_waiting_user, status) in review_status.iter_mut() {
                    let new_status = {
                        let s = mem::replace(status, ReviewStatus::NoReview);
                        match s {
                            ReviewStatus::RFC { ref user } if *user == comment_user => {
                                ReviewStatus::RFCAnswered {
                                    user: comment_user.clone(),
                                }
                            }
                            x => x,
                        }
                    };
                    mem::replace(status, new_status);
                }

                if comment.parent == None {
                    let user_review = review_status
                        .entry(comment_user.clone())
                        .or_insert(ReviewStatus::NoReview);

                    for comment_line in comment.content.raw.lines() {
                        trace!(logger, "Line: {}", comment_line);
                        let mut splitter = comment_line.split_whitespace();
                        if Some("!g") == splitter.next() {
                            while let Some(cmd) = splitter.next() {
                                debug!(logger, "CMD: {}", cmd);
                                match cmd {
                                    vote if re_vote.is_match(vote) => {
                                        *user_review = ReviewStatus::Voted {
                                            vote: cmd.trim_left_matches('\\').parse::<i32>()?,
                                        }
                                    }
                                    "rfc" => if let Some(wait_for_user) = splitter.next() {
                                        debug!(logger, "ARG: {}", wait_for_user);
                                        *user_review = ReviewStatus::RFC {
                                            user: wait_for_user.to_string(),
                                        }
                                    },
                                    unrecognized_cmd => {
                                        warn!(logger, "Unrecognized cmd: {}", unrecognized_cmd)
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ActivityItem::Update { .. } => {
                for (_waiting_user, status) in review_status.iter_mut() {
                    let new_status = {
                        let s = mem::replace(status, ReviewStatus::NoReview);
                        match s {
                            ReviewStatus::Voted { vote } => {
                                ReviewStatus::VoteNeedReevaluation { voted: vote }
                            }
                            x => x,
                        }
                    };
                    mem::replace(status, new_status);
                }
            }
        }
    }

    // ---------------------------------------------------------------------------------------------

    println!("  PR {}: {}", pr.id, pr.title);
    println!("    -- author: {}", pr.author.username);
    println!("    -- link: {}", urls.web_url);
    for (user, status) in &review_status {
        println!("    {}: {:?}", user, status);
    }

    Ok(())
}

fn main() {
    let mut logger = sloggers::terminal::TerminalLoggerBuilder::new();
    logger.level(sloggers::types::Severity::Trace);
    logger.destination(sloggers::terminal::Destination::Stderr);
    let logger = logger.build().unwrap();

    app(&logger).unwrap();
}
