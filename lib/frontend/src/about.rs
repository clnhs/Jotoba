use std::sync::Arc;

//use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse};
use localization::TranslationDict;

use crate::{
    templates, user_settings, {BaseData, Site},
};

/// About page
pub async fn about(
    locale_dict: web::Data<Arc<TranslationDict>>,
    request: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let settings = user_settings::parse(&request);

    //session::init(&session, &settings);

    Ok(HttpResponse::Ok().body(render!(
        templates::base,
        BaseData::new(&locale_dict, settings).with_site(Site::About)
    )))
}
