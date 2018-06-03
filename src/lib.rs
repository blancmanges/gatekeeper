// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate failure;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate sloggers;

pub mod bitbucket;

use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use bitbucket::ActivityItem;
use bitbucket::Approval;
use bitbucket::PullRequest;

use failure::Error;
use regex::Regex;

#[derive(Debug)]
pub struct RepositoryURLs {
    pub api_url: String,
    pub web_url: String,
}

impl RepositoryURLs {
    pub fn new(repo_owner: &str, repo_slug: &str) -> RepositoryURLs {
        RepositoryURLs {
            api_url: format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests",
                repo_owner, repo_slug
            ),
            web_url: format!(
                "https://bitbucket.org/{}/{}/pull-requests",
                repo_owner, repo_slug
            ),
        }
    }

    pub fn with_id(&self, id: u32) -> PullrequestIdURLs {
        PullrequestIdURLs::new(&self, id)
    }
}

#[derive(Debug)]
pub struct PullrequestIdURLs {
    pub api_url: String,
    pub web_url: String,
}

impl PullrequestIdURLs {
    pub fn new(pullrequests_link: &RepositoryURLs, id: u32) -> PullrequestIdURLs {
        let api_url = format!("{}/{}", pullrequests_link.api_url, id);
        let web_url = format!("{}/{}", pullrequests_link.web_url, id);
        PullrequestIdURLs { api_url, web_url }
    }
}

#[derive(PartialEq, Debug)]
pub struct UserCommand {
    pub user: String,
    pub command: String,
}

impl UserCommand {
    pub fn new(user: &str, command: &str) -> UserCommand {
        let command = if command.starts_with("\\+") {
            command.trim_left_matches("\\")
        } else {
            command
        };
        UserCommand {
            user: user.to_string(),
            command: command.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum ReviewStatus {
    NoReview,
    Voted { vote: i32 },
    VoteNeedReevaluation { voted: i32 },
    WantsToReviewAgain { voted: Option<i32> },
    RFC { user: String },
    RFCAnswered { user: String },
}

#[derive(Debug)]
pub struct PullRequestState {
    pub review_status: HashMap<String, ReviewStatus, RandomState>,
    logger: slog::Logger,
    regex_vote: Regex,
    pub urls: PullrequestIdURLs,
    pub pr: PullRequest,
}

impl PullRequestState {
    pub fn from_activity(
        pr: PullRequest,
        activity: Vec<ActivityItem>,
        urls: PullrequestIdURLs,
        logger: &slog::Logger,
    ) -> Result<PullRequestState, Error> {
        let mut pr_state = PullRequestState::new(pr, urls, logger)?;

        for change in activity {
            trace!(pr_state.logger, "Change: {:?}", change);
            match change {
                ActivityItem::Approval {
                    approval: Approval { user },
                } => {
                    let approve_user = user.username.to_string();
                    debug!(pr_state.logger, "User {:?} approves", approve_user);
                    pr_state
                        .review_status
                        .insert(approve_user, ReviewStatus::Voted { vote: 1 });
                }

                ActivityItem::Comment { comment } => {
                    let comment_user = comment.user.username;

                    for status in pr_state.review_status.values_mut() {
                        let should_update = match *status {
                            ReviewStatus::RFC { ref user } if *user == comment_user => true,
                            _ => false,
                        };
                        if should_update {
                            *status = ReviewStatus::RFCAnswered {
                                user: comment_user.clone(),
                            };
                        }
                    }

                    if comment.parent == None {
                        let user_review = pr_state
                            .review_status
                            .entry(comment_user.clone())
                            .or_insert(ReviewStatus::NoReview);

                        for comment_line in comment.content.raw.lines() {
                            trace!(pr_state.logger, "Line: {}", comment_line);
                            let mut splitter = comment_line.split_whitespace();
                            if Some("!g") == splitter.next() {
                                while let Some(cmd) = splitter.next() {
                                    debug!(pr_state.logger, "CMD: {}", cmd);
                                    match cmd {
                                        vote if pr_state.regex_vote.is_match(vote) => {
                                            *user_review = ReviewStatus::Voted {
                                                vote: cmd.trim_left_matches('\\').parse::<i32>()?,
                                            }
                                        }
                                        "rfc" => if let Some(wait_for_user) = splitter.next() {
                                            debug!(pr_state.logger, "ARG: {}", wait_for_user);
                                            *user_review = ReviewStatus::RFC {
                                                user: wait_for_user.to_string(),
                                            }
                                        },
                                        "will\\_revote" => {
                                            let voted = match *user_review {
                                                ReviewStatus::WantsToReviewAgain { voted } => voted,
                                                ReviewStatus::Voted { vote } => Some(vote),
                                                ReviewStatus::VoteNeedReevaluation { voted } => {
                                                    Some(voted)
                                                }
                                                _ => None,
                                            };
                                            *user_review =
                                                ReviewStatus::WantsToReviewAgain { voted }
                                        }
                                        unrecognized_cmd => warn!(
                                            pr_state.logger,
                                            "Unrecognized cmd: {}", unrecognized_cmd
                                        ),
                                    }
                                }
                            }
                        }
                    }
                }

                ActivityItem::Update { .. } => for status in pr_state.review_status.values_mut() {
                    let should_update = match *status {
                        ReviewStatus::Voted { vote } => Some(vote),
                        _ => None,
                    };
                    if let Some(vote) = should_update {
                        *status = ReviewStatus::VoteNeedReevaluation { voted: vote };
                    }
                },
            }
        }

        Ok(pr_state)
    }

    fn new(
        pr: PullRequest,
        urls: PullrequestIdURLs,
        logger: &slog::Logger,
    ) -> Result<PullRequestState, Error> {
        let regex_vote = Regex::new(r"^(\\?\+|-)?\d$")?;
        let logger = logger.new(o!());
        Ok(PullRequestState {
            review_status: HashMap::new(),
            logger,
            regex_vote,
            urls,
            pr,
        })
    }
}
