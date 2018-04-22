// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub mod pull_request_id;

use bb_api::repositories::username::repo_slug::pullrequests::PullRequest;
use bb_api::Paginated;
use serde_json;

pub fn api_page_1() -> &'static str {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    r#"{"pagelen": 10, "values": [{"description": "foo description", "links": {"decline": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/decline"}, "commits": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/commits"}, "self": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1"}, "comments": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/comments"}, "merge": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/merge"}, "html": {"href": "https://bitbucket.org/kgadek/test-repo/pull-requests/1"}, "activity": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/activity"}, "diff": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/diff"}, "approve": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/approve"}, "statuses": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/statuses"}}, "title": "foo title", "close_source_branch": true, "merge_commit": null, "destination": {"commit": {"hash": "04332f0becb5", "links": {"self": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/commit/04332f0becb5"}}}, "repository": {"links": {"self": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo"}, "html": {"href": "https://bitbucket.org/kgadek/test-repo"}, "avatar": {"href": "https://bitbucket.org/kgadek/test-repo/avatar/32/"}}, "type": "repository", "name": "test-repo", "full_name": "kgadek/test-repo", "uuid": "{65d17e2e-94dc-4316-8477-f63130cfb495}"}, "branch": {"name": "master"}}, "state": "OPEN", "closed_by": null, "summary": {"raw": "foo description", "markup": "markdown", "html": "<p>foo description</p>", "type": "rendered"}, "source": {"commit": {"hash": "671832997888", "links": {"self": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/commit/671832997888"}}}, "repository": {"links": {"self": {"href": "https://api.bitbucket.org/2.0/repositories/kgadek/test-repo"}, "html": {"href": "https://bitbucket.org/kgadek/test-repo"}, "avatar": {"href": "https://bitbucket.org/kgadek/test-repo/avatar/32/"}}, "type": "repository", "name": "test-repo", "full_name": "kgadek/test-repo", "uuid": "{65d17e2e-94dc-4316-8477-f63130cfb495}"}, "branch": {"name": "pr-uno"}}, "comment_count": 10, "author": {"username": "kgadek", "display_name": "kgadek", "type": "user", "uuid": "{00eaf6e2-088d-4bd5-90ba-e83dde4b827e}", "links": {"self": {"href": "https://api.bitbucket.org/2.0/users/kgadek"}, "html": {"href": "https://bitbucket.org/kgadek/"}, "avatar": {"href": "https://bitbucket.org/account/kgadek/avatar/32/"}}}, "created_on": "2018-04-01T19:00:22.232192+00:00", "reason": "", "updated_on": "2018-04-07T22:30:43.354657+00:00", "type": "pullrequest", "id": 1, "task_count": 0}], "page": 1, "size": 1}"#
}

#[test]
fn deserialize_api_page_1() {
    let data = api_page_1();
    let v: Result<Paginated<PullRequest>, _> = serde_json::from_str(data);

    assert!(v.is_ok(), "Deserialization returned: {:?}", v);
}
