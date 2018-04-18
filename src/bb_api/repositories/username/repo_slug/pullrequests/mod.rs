// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub mod pull_request_id;

#[derive(Deserialize, PartialEq, Debug)]
pub struct PullRequest {
    pub id: u32,
    pub title: String,
    pub state: String,
}
