// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub mod example_input;

use self::activity::ActivityItem;
use bb_api::repositories::username::repo_slug::pullrequests::pull_request_id::activity;
use bb_api::Paginated;
use serde_json;

#[test]
fn deserialize_basic_1() {
    let data = example_input::api_page_1();
    let v: Result<Paginated<ActivityItem>, _> = serde_json::from_str(data);

    assert!(v.is_ok(), "Deserialization returned: {:?}", v);
}

#[test]
fn deserialize_basic_2() {
    let data = example_input::api_page_2();
    let v: Result<Paginated<ActivityItem>, _> = serde_json::from_str(data);

    assert!(v.is_ok(), "Deserialization returned: {:?}", v);
}

#[test]
fn no_new_fields_allowed() {
    let data = r#"{
        "pagelen": 10,
        "values": [
            {"unknown_marker": {}}
        ]
    }"#;
    let v: Result<Paginated<ActivityItem>, _> = serde_json::from_str(data);
    assert!(v.is_err(), "v should have failed, but got {:?}", v);
}

#[test]
fn deserialize_api_page_1() {
    fn cl() -> Result<(), serde_json::Error> {
        let data = example_input::api_page_1();
        let v: Paginated<ActivityItem> = serde_json::from_str(data)?;

        let next = v.next;
        assert_eq!(Some(String::from("https://api.bitbucket.org/2.0/repositories/kgadek/test-repo/pullrequests/1/activity?ctx=foobar")), next);

        assert_eq!(10, v.values.len());
        for (index, item) in v.values.into_iter().enumerate() {
            match item {
                ActivityItem::Comment { .. } => (),
                _ => assert!(false, "Item number {}", index),
            }
        }

        Ok(())
    };
    let res = cl();
    assert!(res.is_ok(), "Error: {:?}", res);
}

#[test]
fn deserialize_api_page_2() {
    fn cl() -> Result<(), serde_json::Error> {
        let data = example_input::api_page_2();
        let v: Paginated<ActivityItem> = serde_json::from_str(data)?;

        let next = v.next;
        assert_eq!(None, next);

        assert_eq!(4, v.values.len());
        for (index, item) in v.values.into_iter().enumerate() {
            match item {
                ActivityItem::Update { .. } => (),
                _ => assert!(false, "Item number {}", index),
            }
        }

        Ok(())
    };
    let res = cl();
    assert!(res.is_ok(), "Error: {:?}", res);
}
