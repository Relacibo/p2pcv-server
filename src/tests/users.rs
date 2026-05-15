use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};

use crate::db::users::User;

use super::helpers::*;

// ── Users ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_users_is_public() {
    let db = test_db().await;
    let user = create_test_user(&db).await;

    let resp = send(
        test_app(db.clone()),
        Request::builder().uri("/users").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let uid = user.id.to_string();
    assert!(body.as_array().unwrap().iter().any(|u| u["id"] == uid));

    User::delete(&db, user.id).await.unwrap();
}

#[tokio::test]
async fn get_user_requires_auth() {
    let db = test_db().await;
    let user = create_test_user(&db).await;

    let no_auth = send(
        test_app(db.clone()),
        Request::builder()
            .uri(format!("/users/{}", user.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(no_auth.status(), StatusCode::UNAUTHORIZED);

    User::delete(&db, user.id).await.unwrap();
}

#[tokio::test]
async fn get_user_returns_correct_data() {
    let db = test_db().await;
    let user = create_test_user(&db).await;

    let resp = send(
        test_app(db.clone()),
        Request::builder()
            .uri(format!("/users/{}", user.id))
            .header(header::AUTHORIZATION, bearer(user.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["id"].as_str().unwrap(), user.id.to_string());
    assert_eq!(body["userName"].as_str().unwrap(), user.user_name);

    User::delete(&db, user.id).await.unwrap();
}

#[tokio::test]
async fn delete_user_rejects_other_user() {
    let db = test_db().await;
    let user = create_test_user(&db).await;
    let other = create_test_user(&db).await;

    let resp = send(
        test_app(db.clone()),
        Request::builder()
            .method(Method::DELETE)
            .uri(format!("/users/{}", user.id))
            .header(header::AUTHORIZATION, bearer(other.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    User::delete(&db, user.id).await.unwrap();
    User::delete(&db, other.id).await.unwrap();
}

#[tokio::test]
async fn delete_user_succeeds_for_self() {
    let db = test_db().await;
    let user = create_test_user(&db).await;

    let resp = send(
        test_app(db.clone()),
        Request::builder()
            .method(Method::DELETE)
            .uri(format!("/users/{}", user.id))
            .header(header::AUTHORIZATION, bearer(user.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(User::get(&db, user.id).await.is_err());
}

// ── Friend requests & friends ─────────────────────────────────────────────────

#[tokio::test]
async fn friend_request_flow() {
    let db = test_db().await;
    let alice = create_test_user(&db).await;
    let bob = create_test_user(&db).await;

    // Alice sends a friend request to Bob
    let resp = send(
        test_app(db.clone()),
        Request::builder()
            .method(Method::POST)
            .uri(format!(
                "/users/{}/friend-requests/send-to/{}",
                alice.id, bob.id
            ))
            .header(header::AUTHORIZATION, bearer(alice.id))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"message": "hey!"}"#))
            .unwrap(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Bob sees the incoming request
    let resp = send(
        test_app(db.clone()),
        Request::builder()
            .uri(format!("/users/{}/friend-requests/incoming", bob.id))
            .header(header::AUTHORIZATION, bearer(bob.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["friendRequests"].as_array().unwrap().len(), 1);
    assert_eq!(
        body["friendRequests"][0]["sender"]["id"].as_str().unwrap(),
        alice.id.to_string()
    );

    // Bob accepts
    let resp = send(
        test_app(db.clone()),
        Request::builder()
            .method(Method::POST)
            .uri(format!(
                "/users/{}/friend-requests/by-sender/{}/accept",
                bob.id, alice.id
            ))
            .header(header::AUTHORIZATION, bearer(bob.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Bob's friend list now includes Alice
    let resp = send(
        test_app(db.clone()),
        Request::builder()
            .uri(format!("/users/{}/friends", bob.id))
            .header(header::AUTHORIZATION, bearer(bob.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let alice_id = alice.id.to_string();
    assert!(body["friends"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["friend"]["id"] == alice_id));

    User::delete(&db, alice.id).await.unwrap();
    User::delete(&db, bob.id).await.unwrap();
}

#[tokio::test]
async fn duplicate_friend_request_is_rejected() {
    let db = test_db().await;
    let alice = create_test_user(&db).await;
    let bob = create_test_user(&db).await;

    let send_req = || {
        Request::builder()
            .method(Method::POST)
            .uri(format!(
                "/users/{}/friend-requests/send-to/{}",
                alice.id, bob.id
            ))
            .header(header::AUTHORIZATION, bearer(alice.id))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .unwrap()
    };

    let resp = send(test_app(db.clone()), send_req()).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Second request in same direction should fail
    let resp = send(test_app(db.clone()), send_req()).await;
    assert!(resp.status().is_client_error());

    User::delete(&db, alice.id).await.unwrap();
    User::delete(&db, bob.id).await.unwrap();
}
