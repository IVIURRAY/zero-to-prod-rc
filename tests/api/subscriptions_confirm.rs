use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use zero2prod::email_client::Address;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400)
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_links = app.get_confirmation_links(&email_request);

    // Act
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // Act
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}

#[tokio::test]
async fn the_request_structure_for_mailjet_is_valid() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    // Act
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // Assert
    let message = &body["Messages"][0];

    let expected_address = Address {
        email: "mini_muz_11@hotmail.co.uk",
        name: "Me",
    };
    assert_eq!(&expected_address, &message["From"]);

    let mut expected_to_address = vec![];
    let expected_from_address_1 = Address {
        email: "mini_muz_11@hotmail.co.uk",
        name: "Me",
    };
    let expected_from_address_2 = Address {
        email: "ursula_le_guin@gmail.com",
        name: "You",
    };
    expected_to_address.push(expected_from_address_1);
    expected_to_address.push(expected_from_address_2);
    let to_address = &message["To"];
    assert_eq!(&expected_to_address[0], to_address.get(0).unwrap());
    assert_eq!(&expected_to_address[1], to_address.get(1).unwrap());

    assert_eq!("Welcome zero2prod!", &message["Subject"]);

    assert!(&message["HtmlPart"]
        .to_string()
        .contains("Welcome to our newsletter!"));
    assert!(&message["TextPart"]
        .to_string()
        .contains("Welcome to our newsletter!"));
}
