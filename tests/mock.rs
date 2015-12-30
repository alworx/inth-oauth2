extern crate chrono;
extern crate hyper;
extern crate inth_oauth2;
#[macro_use]
extern crate yup_hyper_mock;

use chrono::{UTC, Duration};
use inth_oauth2::{Client, ClientError, Token, Lifetime};

mod provider {
    use inth_oauth2::token::{Bearer, Static, Expiring};
    use inth_oauth2::provider::Provider;

    pub struct BearerStatic;
    impl Provider for BearerStatic {
        type Lifetime = Static;
        type Token = Bearer<Static>;
        fn auth_uri() -> &'static str { "https://example.com/oauth/auth" }
        fn token_uri() -> &'static str { "https://example.com/oauth/token" }
    }

    pub struct BearerExpiring;
    impl Provider for BearerExpiring {
        type Lifetime = Expiring;
        type Token = Bearer<Expiring>;
        fn auth_uri() -> &'static str { "https://example.com/oauth/auth" }
        fn token_uri() -> &'static str { "https://example.com/oauth/token" }
    }
}

mod connector {
    use hyper;

    mock_connector_in_order!(BearerStaticSuccess {
        include_str!("response/request_token_bearer_static_success.http")
    });

    mock_connector_in_order!(BearerExpiringSuccess {
        include_str!("response/request_token_bearer_expiring_success.http")
        include_str!("response/refresh_token_bearer_full.http")
    });

    mock_connector_in_order!(BearerExpiringSuccessPartial {
        include_str!("response/request_token_bearer_expiring_success.http")
        include_str!("response/refresh_token_bearer_partial.http")
    });
}

macro_rules! mock_client {
    ($p:ty, $c:ty) => {
        Client::<$p>::new(
            hyper::Client::with_connector(<$c>::default()),
            "client_id",
            "client_secret",
            None
        )
    }
}


#[test]
fn request_token_bearer_static_success() {
    let client = mock_client!(provider::BearerStatic, connector::BearerStaticSuccess);
    let token = client.request_token("code").unwrap();
    assert_eq!("aaaaaaaa", token.access_token());
    assert_eq!(Some("example"), token.scope());
}

#[test]
fn request_token_bearer_expiring_success() {
    let client = mock_client!(provider::BearerExpiring, connector::BearerExpiringSuccess);
    let token = client.request_token("code").unwrap();
    assert_eq!("aaaaaaaa", token.access_token());
    assert_eq!(Some("example"), token.scope());
    assert_eq!("bbbbbbbb", token.lifetime().refresh_token());
    assert_eq!(false, token.lifetime().expired());
    assert!(token.lifetime().expires() > &UTC::now());
    assert!(token.lifetime().expires() <= &(UTC::now() + Duration::seconds(3600)));
}

#[test]
fn refresh_token_bearer_full() {
    let client = mock_client!(provider::BearerExpiring, connector::BearerExpiringSuccess);
    let token = client.request_token("code").unwrap();
    let token = client.refresh_token(token, None).unwrap();
    assert_eq!("cccccccc", token.access_token());
    assert_eq!(Some("example"), token.scope());
    assert_eq!("dddddddd", token.lifetime().refresh_token());
    assert_eq!(false, token.lifetime().expired());
    assert!(token.lifetime().expires() > &UTC::now());
    assert!(token.lifetime().expires() <= &(UTC::now() + Duration::seconds(3600)));
}

#[test]
fn refresh_token_bearer_partial() {
    let client = mock_client!(provider::BearerExpiring, connector::BearerExpiringSuccessPartial);
    let token = client.request_token("code").unwrap();
    let token = client.refresh_token(token, None).unwrap();
    assert_eq!("cccccccc", token.access_token());
    assert_eq!(Some("example"), token.scope());
    assert_eq!("bbbbbbbb", token.lifetime().refresh_token());
    assert_eq!(false, token.lifetime().expired());
    assert!(token.lifetime().expires() > &UTC::now());
    assert!(token.lifetime().expires() <= &(UTC::now() + Duration::seconds(3600)));
}

#[test]
fn request_token_bearer_static_wrong_lifetime() {
    let client = mock_client!(provider::BearerStatic, connector::BearerExpiringSuccess);
    let err = client.request_token("code").unwrap_err();
    assert!(match err { ClientError::Parse(..) => true, _ => false });
}

#[test]
fn request_token_bearer_expiring_wrong_lifetime() {
    let client = mock_client!(provider::BearerExpiring, connector::BearerStaticSuccess);
    let err = client.request_token("code").unwrap_err();
    assert!(match err { ClientError::Parse(..) => true, _ => false });
}