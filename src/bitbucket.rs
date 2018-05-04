// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::Debug;

use failure::Error;
use reqwest;
use serde;
use serde_json;
use slog;

pub struct BitBucketApiBasicAuth {
    client: reqwest::Client,
    username: String,
    password: String,
}

impl BitBucketApiBasicAuth {
    pub fn new(
        username: String,
        password: String,
        client: reqwest::Client,
    ) -> BitBucketApiBasicAuth {
        BitBucketApiBasicAuth {
            client,
            username,
            password,
        }
    }
    pub fn get_json(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let mut req_builder = self.client.get(url);
        req_builder
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .header(reqwest::header::ContentType::json());
        req_builder.send()
    }
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Paginated<T> {
    pub values: Vec<T>,
    pub next: Option<String>,
}

impl<T> Paginated<T>
where
    T: serde::de::DeserializeOwned + Debug,
{
    pub fn unpaginate(
        self,
        client: &BitBucketApiBasicAuth,
        logger: &slog::Logger,
    ) -> Result<Vec<T>, Error> {
        let mut current = self;

        let mut res: Vec<T> = Vec::new();
        res.append(&mut current.values);

        while let Some(next_url) = current.next {
            debug!(logger, "Requesting next page: {}", next_url);
            let mut result = client.get_json(next_url.as_str())?;
            trace!(logger, "Response: {:?}", result);
            let res_txt = result.text()?;

            current = serde_json::from_str(res_txt.as_str())?;

            trace!(logger, "Response: {:?}", current);
            res.append(&mut current.values);
        }

        Ok(res)
    }
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct PullRequest {
    pub id: u32,
    pub title: String,
    pub state: String,
    pub links: PullRequestLinks,
    pub author: PullRequestUser,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct PullRequestLinks {
    #[serde(rename = "self")]
    pub slf: Href,
    pub activity: Href,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Href {
    pub href: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct PullRequestUser {
    pub username: String,
}

type Ignored = serde_json::value::Value;

#[derive(PartialEq, Debug, Deserialize)]
#[serde(untagged)]
pub enum ActivityItem {
    Comment { comment: Comment },
    Update { update: Ignored },
    Approval { approval: Approval },
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Comment {
    pub id: u32,
    pub parent: Option<CommentParent>,
    pub content: Content,
    pub user: User,
}

impl Comment {
    pub fn is_top_level(&self) -> bool {
        self.parent == None
    }
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct CommentParent {
    pub id: u32,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Content {
    pub raw: String,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Approval {
    pub user: User,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct User {
    pub username: String,
}

pub fn get_all<T>(
    url: &str,
    client: &BitBucketApiBasicAuth,
    logger: &slog::Logger,
) -> Result<Vec<T>, Error>
where
    T: serde::de::DeserializeOwned + Debug,
{
    let logger = logger.new(o!(
        "url" => url.to_string(),
    ));

    trace!(logger, "Obtaining first page");
    let mut first_page = client.get_json(url)?;
    let first_page = first_page.text()?;
    trace!(logger, "Response text: {}", first_page);
    trace!(logger, "Deserializing");
    let first_page: Paginated<T> = serde_json::from_str(first_page.as_str())?;

    trace!(logger, "Getting remaining pages");
    first_page.unpaginate(&client, &logger)
}
