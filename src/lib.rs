// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod bb_api;
#[cfg(test)]
mod tests;

use self::activity::{Activity, ActivityItem, Comment};
use bb_api::repositories::username::repo_slug::pullrequests::pull_request_id::activity;

pub fn activities_to_items(act: Vec<Activity>) -> Vec<ActivityItem> {
    let mut res = Vec::new();
    for mut item in act {
        res.append(&mut item.values);
    }
    res
}

pub fn toplevel_comments(act_items: Vec<ActivityItem>) -> Vec<Comment> {
    let mut res = Vec::new();
    for mut item in act_items {
        match item {
            ActivityItem::Comment {
                comment: comment @ Comment { parent: None, .. },
            } => res.push(comment),
            _ => (),
        }
    }
    res
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
    let mut res = Vec::new();
    for mut comment in toplevel_comments(act_items) {
        for comment_line in comment.content.raw.lines() {
            let mut splitter = comment_line.split_whitespace();
            if Some("@g") == splitter.next() {
                for command in splitter {
                    res.push(UserCommand {
                        user: comment.user.username.clone(),
                        command: command.to_string(),
                    })
                }
            }
        }
    }
    res
}
