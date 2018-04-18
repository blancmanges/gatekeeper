// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate serde;
extern crate serde_json;

mod bb_api;

use self::activity::{ActivityItem, Comment, CommentContent, CommentParent, CommentUser};
use super::*;
use bb_api::repositories::username::repo_slug::pullrequests::pull_request_id::activity;
use serde_json::value::Value;

#[test]
fn activities_to_items_empty() {
    let inp: Vec<Paginated<ActivityItem>> = vec![];
    let out: Vec<ActivityItem> = vec![];
    assert_eq!(out, activities_to_items(inp));
}

fn mk_comment_1() -> Comment {
    Comment {
        id: 123,
        parent: None,
        content: CommentContent {
            raw: r#"comment1-raw
            Another line. 𮯛
            @g command11
            Yet 🎅 another line


            "#.to_string(),
        },
        user: CommentUser {
            username: "comment1-user".to_string(),
        },
    }
}

fn mk_comment_2() -> Comment {
    Comment {
        id: 456,
        parent: Some(CommentParent { id: 123 }),
        content: CommentContent {
            raw: r#"comment2-raw
            @g command21"#.to_string(),
        },
        user: CommentUser {
            username: "comment2-user".to_string(),
        },
    }
}

fn mk_comment_3() -> Comment {
    Comment {
        id: 789,
        parent: None,
        content: CommentContent {
            raw: r#"comment3-raw first line.
            @g command31 command32
            Another line... 𠅻𠒯

            @g command33"#.to_string(),
        },
        user: CommentUser {
            username: "comment3-user".to_string(),
        },
    }
}

#[test]
fn activities_to_items_multi_nonempty() {
    let inp: Vec<Paginated<ActivityItem>> = vec![
        Paginated {
            pagelen: 3,
            next: Some("foo".to_string()),
            values: vec![
                ActivityItem::Comment {
                    comment: mk_comment_1(),
                },
                ActivityItem::Update {
                    update: Value::Null,
                },
                ActivityItem::Comment {
                    comment: mk_comment_1(),
                },
            ],
        },
        Paginated {
            pagelen: 3,
            next: None,
            values: vec![
                ActivityItem::Update {
                    update: Value::Null,
                },
                ActivityItem::Comment {
                    comment: mk_comment_2(),
                },
                ActivityItem::Comment {
                    comment: mk_comment_3(),
                },
            ],
        },
    ];
    let out: Vec<ActivityItem> = vec![
        ActivityItem::Comment {
            comment: mk_comment_1(),
        },
        ActivityItem::Update {
            update: Value::Null,
        },
        ActivityItem::Comment {
            comment: mk_comment_1(),
        },
        ActivityItem::Update {
            update: Value::Null,
        },
        ActivityItem::Comment {
            comment: mk_comment_2(),
        },
        ActivityItem::Comment {
            comment: mk_comment_3(),
        },
    ];

    assert_eq!(out, activities_to_items(inp));
}

#[test]
fn toplevel_comments_empty() {
    let inp: Vec<ActivityItem> = vec![];
    let out: Vec<Comment> = vec![];

    assert_eq!(out, toplevel_comments(inp));
}

#[test]
fn toplevel_comments_single() {
    let inp: Vec<ActivityItem> = vec![
        ActivityItem::Comment {
            comment: mk_comment_1(),
        },
    ];
    let out: Vec<Comment> = vec![mk_comment_1()];

    assert_eq!(out, toplevel_comments(inp));
}

#[test]
fn toplevel_comments_multi() {
    let inp: Vec<ActivityItem> = vec![
        ActivityItem::Comment {
            comment: mk_comment_1(),
        },
        ActivityItem::Comment {
            comment: mk_comment_2(),
        },
        ActivityItem::Comment {
            comment: mk_comment_3(),
        },
    ];
    let out: Vec<Comment> = vec![mk_comment_1(), mk_comment_3()];

    assert_eq!(out, toplevel_comments(inp));
}

#[test]
fn get_commands_multi() {
    let inp: Vec<ActivityItem> = vec![
        ActivityItem::Comment {
            comment: mk_comment_1(),
        },
        ActivityItem::Comment {
            comment: mk_comment_2(),
        },
        ActivityItem::Comment {
            comment: mk_comment_3(),
        },
    ];
    let out: Vec<UserCommand> = vec![
        UserCommand::new("comment1-user", "command11"),
        UserCommand::new("comment3-user", "command31"),
        UserCommand::new("comment3-user", "command32"),
        UserCommand::new("comment3-user", "command33"),
    ];

    assert_eq!(out, get_commands(inp));
}
