// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate sloggers;

pub mod bitbucket;

use bitbucket::ActivityItem;
use bitbucket::Approval;
use bitbucket::Comment;

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

pub fn toplevel_comments(act_items: Vec<ActivityItem>) -> Vec<Comment> {
    act_items
        .into_iter()
        .filter_map(|item| match item {
            ActivityItem::Comment { comment } => if comment.is_top_level() {
                Some(comment)
            } else {
                None
            },
            _ => None,
        })
        .collect()
}

pub fn get_commands(act_items: Vec<ActivityItem>) -> Vec<UserCommand> {
    let mut commands = Vec::new();

    for mut activity in act_items {
        match activity {
            ActivityItem::Comment {
                comment: comment @ Comment { parent: None, .. },
            } => for comment_line in comment.content.raw.lines() {
                let mut splitter = comment_line.split_whitespace();
                if Some("!g") == splitter.next() {
                    for command in splitter {
                        commands.push(UserCommand::new(&comment.user.username, command))
                    }
                }
            },
            ActivityItem::Approval {
                approval: Approval { user },
            } => commands.push(UserCommand {
                user: user.username.clone(),
                command: "+1".to_string(),
            }),
            _ => {}
        }
    }

    commands
}
