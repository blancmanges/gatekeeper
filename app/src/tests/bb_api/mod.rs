// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod repositories;

use self::activity::ActivityItem;
use self::repositories::username::repo_slug::pullrequests::pull_request_id::activity::example_input;
use bb_api::repositories::username::repo_slug::pullrequests::pull_request_id::activity;
use bb_api::Paginated;
use serde_json;

#[test]
fn deserialize_paginated() {
    let data = example_input::api_page_1();
    let v: Result<Paginated<ActivityItem>, _> = serde_json::from_str(data);

    assert!(v.is_ok());
}
