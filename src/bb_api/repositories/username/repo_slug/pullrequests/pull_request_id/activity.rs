// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde_json::value::Value;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Activity {
    pub pagelen: u32,
    pub next: Option<String>,
    pub values: Vec<ActivityItem>,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(untagged)]
pub enum ActivityItem {
    Comment { comment: Comment },
    Update { update: Value },
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Comment {
    pub id: u32,
    pub parent: Option<CommentParent>,
    pub content: CommentContent,
    pub user: CommentUser,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct CommentParent {
    pub id: u32,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct CommentContent {
    pub raw: String,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct CommentUser {
    pub username: String,
}
