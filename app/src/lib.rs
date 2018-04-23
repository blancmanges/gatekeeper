// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate sloggers;

pub mod bb_api;
#[cfg(test)]
mod tests;

use self::activity::{ActivityItem, Comment};
use bb_api::repositories::username::repo_slug::pullrequests::pull_request_id::activity;
use bb_api::Paginated;
use std::fmt::Debug;
use bb_api::repositories::username::repo_slug::pullrequests::pull_request_id::activity::Approval;

pub fn activities_to_items(act: Vec<Paginated<ActivityItem>>) -> Vec<ActivityItem> {
    let mut res = Vec::new();
    for mut item in act {
        res.append(&mut item.values);
    }
    res
}

pub fn toplevel_comments(act_items: Vec<ActivityItem>) -> Vec<Comment> {
    let mut activities = Vec::new();
    for mut item in act_items {
        match item {
            ActivityItem::Comment {
                comment: comment @ Comment { parent: None, .. },
            } => activities.push(comment),
            _ => (),
        }
    }
    activities
}

#[derive(PartialEq, Debug)]
pub struct UserCommand {
    pub user: String,
    pub command: String,
}

impl UserCommand {
    pub fn new(user: &str, command: &str) -> UserCommand {
        UserCommand {
            user: user.to_string(),
            command: command.to_string(),
        }
    }
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
                        commands.push(UserCommand {
                            user: comment.user.username.clone(),
                            command: command.to_string(),
                        })
                    }
                }
            },
            ActivityItem::Approval {
                approval: Approval { user },
            } => commands.push(UserCommand {
                user: user.username.clone(),
                command: "\\+1".to_string(),
            }),
            _ => {}
        }
    }

    commands
}

pub fn unpaginate<T, F>(
    mut current: Paginated<T>,
    reqwest_get: F,
    logger: &slog::Logger,
) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned + Debug,
    F: for<'a> Fn(&'a str) -> reqwest::RequestBuilder,
{
    let mut res: Vec<T> = Vec::new();
    res.append(&mut current.values);

    while let Some(next_url) = current.next {
        debug!(logger, "Requesting next page: {}", next_url);
        let mut result = reqwest_get(next_url.as_str())
            .send()
            .map_err(|e| format!("{}", e))?;
        trace!(logger, "Response: {:?}", result);
        let res_txt = result.text().map_err(|e| e.to_string())?;

        current = serde_json::from_str(res_txt.as_str()).map_err(|e| e.to_string())?;
        trace!(logger, "Deserialized response: {:?}", current);
        res.append(&mut current.values);
    }

    Ok(res)
}
