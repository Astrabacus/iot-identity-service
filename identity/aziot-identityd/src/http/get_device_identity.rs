// Copyright (c) Microsoft. All rights reserved.

pub(super) fn handle(
    req: hyper::Request<hyper::Body>,
    inner: std::sync::Arc<futures_util::lock::Mutex<aziot_identityd::Server>>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<hyper::Response<hyper::Body>, hyper::Request<hyper::Body>>> + Send>> {
    Box::pin(async move {
        if req.uri().path() != "/identities/device" {
            return Err(req);
        }

        let mut inner = inner.lock().await;
		let inner = &mut *inner;

        let user = aziot_identityd::auth::Uid(0);
        let auth_id = match inner.authenticator.authenticate(user) {
            Ok(auth_id) => auth_id,
            Err(err) => return Ok(super::ToHttpResponse::to_http_response(&err)),
        };

        let (http::request::Parts { method, .. }, body) = req.into_parts();

        if method != hyper::Method::POST {
            return Ok(super::err_response(
                hyper::StatusCode::METHOD_NOT_ALLOWED,
                Some((hyper::header::ALLOW, "POST")),
                "method not allowed".into(),
            ));
        }

        let body = match hyper::body::to_bytes(body).await {
            Ok(body) => body,
            Err(err) => return Ok(super::err_response(
                hyper::StatusCode::BAD_REQUEST,
                None,
                super::error_to_message(&err).into(),
            )),
        };

        let body: aziot_identity_common_http::get_device_identity::Request = match serde_json::from_slice(&body) {
            Ok(body) => body,
            Err(err) => return Ok(super::err_response(
                hyper::StatusCode::UNPROCESSABLE_ENTITY,
                None,
                super::error_to_message(&err).into(),
            )),
        };

        //TODO: get uid from UDS
        let response = match inner.get_device_identity(auth_id,&body.id_type).await {
            Ok(v) => v,
            Err(err) => return Ok(super::ToHttpResponse::to_http_response(&err)),
        };
        let response = aziot_identity_common_http::get_device_identity::Response { identity: response };

        let response = super::json_response(hyper::StatusCode::OK, &response);
        Ok(response)
    })
}